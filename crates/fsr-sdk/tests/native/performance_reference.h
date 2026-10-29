// SPDX-License-Identifier: MPL-2.0

// Independent minimal consumer implementation: uses pinned AMD headers, never Rust.
#define NOMINMAX
#include "cpu_equipment.h"
#include "ffx_api_dx12.h"
#include "ffx_upscale.h"
#include <wrl/client.h>
#include <array>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <string>
#include <vector>

using Microsoft::WRL::ComPtr;

static void require(bool success, const char* operation) {
    if (!success) {
        std::fprintf(stderr, "CPU reference failure: %s\n", operation);
        std::exit(1); // Never submit failed recordings or unwind in-flight state.
    }
}

static std::wstring environment(const wchar_t* name) {
    const DWORD length=GetEnvironmentVariableW(name,nullptr,0);
    require(length>1,"environment value");
    std::vector<wchar_t> buffer(length);
    require(GetEnvironmentVariableW(name,buffer.data(),length)==length-1,"environment read");
    return std::wstring(buffer.data());
}

template<class T> static T symbol(HMODULE module, const char* name) {
    const auto pointer = GetProcAddress(module, name);
    require(pointer != nullptr, name);
    T result{};
    static_assert(sizeof(result) == sizeof(pointer));
    std::memcpy(&result, &pointer, sizeof(result));
    return result;
}

static uint32_t bits(float value) {
    uint32_t result{};
    std::memcpy(&result, &value, sizeof(result));
    return result;
}

static void spec(const ffxCreateContextDescUpscale& root,
                 const ffxCreateContextDescUpscaleVersion& version,
                 const ffxCreateBackendDX12Desc& backend,
                 const ffxDispatchDescUpscale& d) {
    std::fprintf(stderr, "CPU_SPEC {\"creation\":[%llu,%llu,%u,%llu,%u,%u,%u,%u,%u,%u],",
        static_cast<unsigned long long>(root.header.type), static_cast<unsigned long long>(version.header.type),
        version.version, static_cast<unsigned long long>(backend.header.type), root.flags,
        root.maxRenderSize.width, root.maxRenderSize.height, root.maxUpscaleSize.width,
        root.maxUpscaleSize.height, root.fpMessage ? 1u : 0u);
    std::fprintf(stderr, "\"shape\":[%llu,%u,%u,%u,%u,%u,%u,%u,%u,%u],\"float_bits\":[",
        static_cast<unsigned long long>(d.header.type), d.header.pNext ? 1u : 0u, d.commandList ? 1u : 0u,
        d.renderSize.width,d.renderSize.height,d.upscaleSize.width,d.upscaleSize.height,
        d.enableSharpening ? 1u : 0u,d.reset ? 1u : 0u,d.flags);
    const float values[] = {d.jitterOffset.x,d.jitterOffset.y,d.motionVectorScale.x,d.motionVectorScale.y,
        d.sharpness,d.frameTimeDelta,d.preExposure,d.cameraNear,d.cameraFar,d.cameraFovAngleVertical,d.viewSpaceToMetersFactor};
    for (size_t i=0; i<std::size(values); ++i) std::fprintf(stderr, "%s%u", i ? "," : "", bits(values[i]));
    std::fprintf(stderr, "],\"resources\":[");
    const FfxApiResource* resources[] = {&d.color,&d.depth,&d.motionVectors,&d.exposure,&d.reactive,&d.transparencyAndComposition,&d.output};
    for (size_t i=0; i<std::size(resources); ++i) {
        const auto& r=*resources[i]; const auto& m=r.description;
        std::fprintf(stderr, "%s[%u,%u,%u,%u,%u,%u,%u,%u,%u,%u]",i ? "," : "",r.resource ? 1u : 0u,
            m.type,m.format,m.width,m.height,m.depth,m.mipCount,m.flags,m.usage,r.state);
    }
    std::fprintf(stderr,"]}\n");
}

int main() {
    const bool debug=environment(L"FSR_SDK_PERFORMANCE_DEBUG")==L"1";
    const auto characterization=GetEnvironmentVariableW(L"FSR_SDK_CHARACTERIZATION",nullptr,0)
        ? environment(L"FSR_SDK_CHARACTERIZATION") : L"";
    require(characterization.empty() || characterization==L"timing" || characterization==L"lifecycle","characterisation mode");
    const auto active_mode=GetEnvironmentVariableW(L"FSR_SDK_ACTIVE_TIMING",nullptr,0)
        ? environment(L"FSR_SDK_ACTIVE_TIMING") : L"";
    const bool active=!active_mode.empty();
    const auto feature=GetEnvironmentVariableW(L"FSR_SDK_FEATURE_TIMING",nullptr,0)
        ? environment(L"FSR_SDK_FEATURE_TIMING") : L"";
    require(feature.empty() || (active_mode==L"max" && (feature==L"masks" || feature==L"hdr")),"feature mode");
    require(!active || (gpu_timing && (active_mode==L"max" || active_mode==L"small" || active_mode==L"transition")),"active mode");
    if (debug) require(gpu_enable_debug()==0,"debug layer");
    ComPtr<ID3D12Device> device;
    require(probe_create_device(&device)==0,"device");
    require(gpu_require_amd(device.Get())==0,"AMD hardware");
    const auto path=environment(L"FSR_SDK_TEST_DLL");
    const auto load_start=performance_clock();
    const HMODULE library=LoadLibraryW(path.c_str());
    require(library != nullptr,"loader");
    const auto create=symbol<PfnFfxCreateContext>(library,"ffxCreateContext");
    const auto dispatch=symbol<PfnFfxDispatch>(library,"ffxDispatch");
    const auto query=symbol<PfnFfxQuery>(library,"ffxQuery");
    const auto destroy=symbol<PfnFfxDestroyContext>(library,"ffxDestroyContext");
    const auto load_ticks=performance_clock()-load_start;
    uint32_t rw=320, rh=180;
    const uint32_t ow=gpu_timing ? 1920u : 640u, oh=gpu_timing ? 1080u : 360u;
    const auto query_start=performance_clock();
#ifdef FSR_GPU_TIMING
    {
        ffxCreateBackendDX12Desc device_query{};
        device_query.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12;
        device_query.device=device.Get();
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode resolution{};
        resolution.header={FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE,&device_query.header};
        resolution.displayWidth=ow; resolution.displayHeight=oh;
        resolution.qualityMode=FFX_UPSCALE_QUALITY_MODE_QUALITY;
        resolution.pOutRenderWidth=&rw; resolution.pOutRenderHeight=&rh;
        require(query(nullptr,&resolution.header)==0,"pre-create render query");
        require(gpu_timing_configure(rw,rh)==0,"configure dimensions");
    }
#endif
    const auto query_ticks=performance_clock()-query_start;
    void* equipment=nullptr;
    ComPtr<ID3D12GraphicsCommandList> list;
    ID3D12Resource* adopted[5]{};
    require(gpu_create(device.Get(),&equipment,&list,adopted)==0,"resources");
    ComPtr<ID3D12Resource> resources[5];
    for (size_t i=0;i<5;++i) resources[i].Attach(adopted[i]);
    ComPtr<ID3D12Resource> masks[2];
#ifdef FSR_GPU_TIMING
    if (!feature.empty()) require(feature_timing_configure(equipment,feature==L"masks" ? 1u : 2u)==0,"feature equipment");
    if (feature==L"masks") {
        ID3D12Resource* adopted_masks[2]{};
        require(gpu_create_masks(equipment,rw,rh,0,0,0,adopted_masks)==0,"mask resources");
        for (size_t i=0;i<2;++i) masks[i].Attach(adopted_masks[i]);
    }
#endif
    CpuMetrics metrics{};
    require(performance_finish(equipment,1,&metrics)==0,"sentinel fence");
    require(metrics.changed==0,"untouched sentinel");

    const auto create_start=performance_clock();
    // These three stack descriptors remain address-stable through destruction.
    ffxCreateBackendDX12Desc backend{};
    backend.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12;
    backend.device=device.Get();
    ffxCreateContextDescUpscaleVersion version{};
    version.header={FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION,&backend.header};
    version.version=FFX_UPSCALER_VERSION;
    ffxCreateContextDescUpscale root{};
    root.header={FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,&version.header};
    root.maxRenderSize={rw,rh}; root.maxUpscaleSize={ow,oh};
    if (active) root.flags=FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION;
    if (feature==L"hdr") root.flags|=FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE;
    ffxContext context=nullptr;
    require(create(&context,&root.header,nullptr)==0 && context,"create context");
    const auto create_ticks=performance_clock()-create_start;
    ffxQueryGetProviderVersion provider{};
    provider.header.type=FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION;
    require(query(&context,&provider.header)==0 && provider.versionId && provider.versionName,"provider query");
    std::fprintf(stderr,"CPU_PROVIDER %llu %s\n",static_cast<unsigned long long>(provider.versionId),provider.versionName);
    std::array<float,160> jx{},jy{};
    std::array<uint32_t,160> widths{},heights{};
    std::array<int32_t,160> phases{};
    widths.fill(rw); heights.fill(rh);
    uint32_t sw=rw,sh=rh;
    int32_t small_phase=0;
    const size_t frames=debug ? 4u : 160u;
#ifdef FSR_GPU_TIMING
    {
        int32_t phase=0;
        uint32_t live_w=0,live_h=0;
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode resolution{};
        resolution.header.type=FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE;
        resolution.displayWidth=ow; resolution.displayHeight=oh;
        resolution.qualityMode=FFX_UPSCALE_QUALITY_MODE_QUALITY;
        resolution.pOutRenderWidth=&live_w; resolution.pOutRenderHeight=&live_h;
        require(query(&context,&resolution.header)==0 && live_w==rw && live_h==rh,"live render query");
        ffxQueryDescUpscaleGetJitterPhaseCount count{};
        count.header.type=FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTERPHASECOUNT;
        count.renderWidth=rw; count.displayWidth=ow; count.pOutPhaseCount=&phase;
        require(query(&context,&count.header)==0 && phase>0,"phase query");
        if (active) {
            resolution.qualityMode=FFX_UPSCALE_QUALITY_MODE_PERFORMANCE;
            resolution.pOutRenderWidth=&sw; resolution.pOutRenderHeight=&sh;
            require(query(&context,&resolution.header)==0 && sw>0 && sh>0 && sw<rw && sh<rh,"small render query");
            count.renderWidth=sw; count.pOutPhaseCount=&small_phase;
            require(query(&context,&count.header)==0 && small_phase>0,"small phase query");
        }
        for (size_t frame=0;frame<frames;++frame) {
            const bool use_small=active_mode==L"small" || (active_mode==L"transition" &&
                (debug ? (frame>=1 && frame<3) : (frame>=64 && frame<112)));
            widths[frame]=use_small ? sw : rw; heights[frame]=use_small ? sh : rh;
            phases[frame]=use_small ? small_phase : phase;
            ffxQueryDescUpscaleGetJitterOffset offset{};
            offset.header.type=FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTEROFFSET;
            offset.index=static_cast<int32_t>(frame)%phases[frame]; offset.phaseCount=phases[frame];
            offset.pOutX=&jx[frame]; offset.pOutY=&jy[frame];
            require(query(&context,&offset.header)==0,"jitter query");
        }
        std::fprintf(stderr,"GPU_QUERY %u %u %d\n",rw,rh,phase);
    }
#endif
    ffxDispatchDescUpscale d{};
    d.header.type=FFX_API_DISPATCH_DESC_TYPE_UPSCALE;
    d.commandList=list.Get();
    d.color=ffxApiGetResourceDX12(resources[0].Get());
    d.depth=ffxApiGetResourceDX12(resources[1].Get());
    d.motionVectors=ffxApiGetResourceDX12(resources[2].Get());
    d.exposure=ffxApiGetResourceDX12(resources[3].Get());
    d.output=ffxApiGetResourceDX12(resources[4].Get(),FFX_API_RESOURCE_STATE_UNORDERED_ACCESS);
    if (feature==L"masks") {
        d.reactive=ffxApiGetResourceDX12(masks[0].Get());
        d.transparencyAndComposition=ffxApiGetResourceDX12(masks[1].Get());
    }
    d.motionVectorScale={static_cast<float>(rw),static_cast<float>(rh)}; d.renderSize={rw,rh}; d.upscaleSize={ow,oh};
    d.frameTimeDelta=16.667f; d.preExposure=1.f; d.reset=true;
    d.cameraNear=.1f; d.cameraFar=100.f; d.cameraFovAngleVertical=1.0471975803375244140625f;
    d.viewSpaceToMetersFactor=1.f;
    spec(root,version,backend,d);

    const auto frequency=performance_frequency();
    require(frequency>0,"QPC frequency");
    std::array<int64_t,256> empty{};
    for (auto& value:empty) {
        const auto start=performance_clock(); const auto end=performance_clock();
        require(start>=0 && end>=start,"empty timer"); value=end-start;
    }
    std::array<int64_t,160> ticks{};
    std::array<std::array<uint64_t,5>,160> gpu_ticks{};
    std::array<std::array<uint64_t,2>,160> hashes{};
    std::array<CpuMetrics,160> checkpoint_metrics{};
    std::array<bool,160> readbacks{};
    for (size_t frame=0;frame<frames;++frame) {
        const bool readback=frame==frames-1 || (active &&
            (debug || frame==31 || frame==63 || frame==64 || frame==111 || frame==112));
        readbacks[frame]=readback;
        d.reset=frame==0;
        d.jitterOffset={jx[frame],jy[frame]};
        if (active) {
            d.renderSize={widths[frame],heights[frame]};
            d.motionVectorScale={static_cast<float>(widths[frame]),static_cast<float>(heights[frame])};
        }
#ifdef FSR_GPU_TIMING
        require((active ? active_timing_begin(equipment,static_cast<uint32_t>(frame),widths[frame],heights[frame],jx[frame],jy[frame],readback ? 1u : 0u)
            : gpu_timing_begin(equipment,static_cast<uint32_t>(frame),jx[frame],jy[frame],readback ? 1u : 0u))==0,"GPU begin");
#else
        if (frame==frames-1) require(performance_prepare_readback(equipment)==0,"prepare output");
#endif
        const auto start=performance_clock();
        const auto code=dispatch(&context,&d.header);
        const auto end=performance_clock();
        require(code==0,"dispatch (list must not be submitted on error)");
#ifdef FSR_GPU_TIMING
        require(gpu_timing_finish(equipment,readback ? 1u : 0u,&metrics,gpu_ticks[frame].data())==0,"GPU finish/fence");
        if (active && readback) {
            require(metrics.finite==ow*oh && metrics.changed==ow*oh && !metrics.removed && (!debug || !metrics.errors),"checkpoint output");
            require(active_timing_hashes(equipment,hashes[frame].data())==0,"checkpoint hashes");
            checkpoint_metrics[frame]=metrics;
        }
#else
        require(performance_finish(equipment,frame==frames-1 ? 1u : 0u,&metrics)==0,"submission/fence");
#endif
        require(start>=0 && end>=start,"timer"); ticks[frame]=end-start;
    }
    require(metrics.width==ow && metrics.height==oh && metrics.finite==ow*oh && metrics.changed==ow*oh && metrics.removed==0,"final output");
    if (debug) require(metrics.errors==0,"debug errors");
    std::vector<uint8_t> payload(ow*oh*8);
    require(gpu_copy_payload(equipment,payload.data(),static_cast<uint32_t>(payload.size()))==0,"copy output");
    const auto output_path=environment(L"FSR_SDK_CPU_OUTPUT");
    std::ofstream output(std::filesystem::path(output_path),std::ios::binary);
    output.write(reinterpret_cast<const char*>(payload.data()),static_cast<std::streamsize>(payload.size()));
    output.close(); require(static_cast<bool>(output),"write output");
    const auto destroy_start=performance_clock();
    require(destroy(&context,nullptr)==0,"destroy context");
    const auto destroy_ticks=performance_clock()-destroy_start;
    if (characterization==L"lifecycle") {
        std::fprintf(stderr,"CHAR_LIFECYCLE load 0 %lld\n",static_cast<long long>(load_ticks));
        std::fprintf(stderr,"CHAR_LIFECYCLE provider_query 0 %lld\n",static_cast<long long>(query_ticks));
        std::fprintf(stderr,"CHAR_LIFECYCLE first_create 0 %lld\n",static_cast<long long>(create_ticks));
        std::fprintf(stderr,"CHAR_LIFECYCLE used_destroy 0 %lld\n",static_cast<long long>(destroy_ticks));
        for (size_t sample=0;sample<8;++sample) {
            context=nullptr;
            const auto start=performance_clock();
            require(create(&context,&root.header,nullptr)==0 && context,"warm create");
            const auto created=performance_clock();
            require(destroy(&context,nullptr)==0,"unused destroy");
            const auto destroyed=performance_clock();
            std::fprintf(stderr,"CHAR_LIFECYCLE warm_create %zu %lld\n",sample,static_cast<long long>(created-start));
            std::fprintf(stderr,"CHAR_LIFECYCLE unused_destroy %zu %lld\n",sample,static_cast<long long>(destroyed-created));
        }
    }
    if (!characterization.empty()) std::fprintf(stderr,"CHAR_MODE %ls\n",characterization.c_str());
    for (auto& resource:resources) resource.Reset();
    for (auto& mask:masks) mask.Reset();
    list.Reset(); gpu_destroy(equipment);
    require((debug ? gpu_check_device(device.Get()) : gpu_check_device_removed(device.Get()))==0,"device checkpoint");
    require(FreeLibrary(library)!=0,"unload");
    std::fprintf(stderr,"CPU_CLOCK %lld\n",static_cast<long long>(frequency));
    for (size_t i=0;i<empty.size();++i) std::fprintf(stderr,"CPU_EMPTY %zu %lld\n",i,static_cast<long long>(empty[i]));
    for (size_t i=0;i<frames;++i) std::fprintf(stderr,"CPU_FRAME %zu %lld\n",i,static_cast<long long>(ticks[i]));
    if (gpu_timing) for (size_t i=0;i<frames;++i) {
        const auto& t=gpu_ticks[i];
        std::fprintf(stderr,"GPU_FRAME %zu %llu %llu %llu %llu %llu\n",i,
            static_cast<unsigned long long>(t[0]),static_cast<unsigned long long>(t[1]),
            static_cast<unsigned long long>(t[2]),static_cast<unsigned long long>(t[3]),static_cast<unsigned long long>(t[4]));
        std::fprintf(stderr,"GPU_JITTER %zu %u %u\n",i,bits(jx[i]),bits(jy[i]));
    }
    if (active) {
        std::fprintf(stderr,"ACTIVE_QUERY %ls %u %u %d\n",active_mode.c_str(),sw,sh,small_phase);
        for (size_t i=0;i<frames;++i) {
            std::fprintf(stderr,"ACTIVE_FRAME %zu %u %u %d %u %u %u\n",i,widths[i],heights[i],phases[i],i==0 ? 1u : 0u,
                bits(static_cast<float>(widths[i])),bits(static_cast<float>(heights[i])));
            if (readbacks[i]) {
                const auto& m=checkpoint_metrics[i]; const auto& h=hashes[i];
                std::fprintf(stderr,"ACTIVE_VALID %zu %u %u %llu %llu %llu\n",i,m.finite,m.changed,
                    static_cast<unsigned long long>(m.errors),static_cast<unsigned long long>(h[0]),static_cast<unsigned long long>(h[1]));
            }
        }
    }
    std::fprintf(stderr,"CPU_VALID sentinel=0 finite=%u written=%u removed=0 debug=%s preinspect=false\n",metrics.finite,metrics.changed,debug ? "true" : "false");
    std::fprintf(stderr,"CPU_COMPARISON PASSED cpp-minimal\n");
    return 0;
}
