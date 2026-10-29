// SPDX-License-Identifier: MPL-2.0

// Performance-only reuse, called after one successful dispatch. All submission,
// waiting, allocator/list recycling and optional readback are outside CPU timing.
extern "C" __declspec(dllexport) int64_t __cdecl performance_clock() {
    LARGE_INTEGER value{};
    return QueryPerformanceCounter(&value) ? value.QuadPart : -1;
}

extern "C" __declspec(dllexport) int64_t __cdecl performance_frequency() {
    LARGE_INTEGER value{};
    return QueryPerformanceFrequency(&value) ? value.QuadPart : -1;
}

extern "C" __declspec(dllexport) int32_t __cdecl performance_prepare_readback(void* handle) {
    if (!handle) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    // Restore the original sentinel before each arm's final recording so stale
    // output from the other context cannot satisfy the written-pixel check.
    const auto desc = texture_desc(Output);
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
    h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
    transition(h.list.Get(), h.textures[Output].Get(),
               D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_STATE_COPY_DEST);
    D3D12_TEXTURE_COPY_LOCATION target{};
    target.pResource = h.textures[Output].Get();
    target.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    D3D12_TEXTURE_COPY_LOCATION source{};
    source.pResource = h.uploads[Output].Get();
    source.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    source.PlacedFootprint = footprint;
    h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
    transition(h.list.Get(), h.textures[Output].Get(),
               D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl performance_finish(
    void* handle, uint32_t readback, Metrics* metrics) {
    if (!handle || readback > 1 || !metrics) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    HRESULT hr = S_OK;
    if (readback) {
        hr = record_readback(h);
        if (FAILED(hr)) return hr;
        transition(h.list.Get(), h.textures[Output].Get(),
                   D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
    }
    hr = h.list->Close();
    if (FAILED(hr)) return hr;
    ID3D12CommandList* lists[] = {h.list.Get()};
    h.queue->ExecuteCommandLists(1, lists);
    const UINT64 value = ++h.performance_fence;
    hr = h.queue->Signal(h.fence.Get(), value);
    if (FAILED(hr)) return hr;
    HANDLE event = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    if (!event) return HRESULT_FROM_WIN32(GetLastError());
    hr = h.fence->SetEventOnCompletion(value, event);
    if (SUCCEEDED(hr) && WaitForSingleObject(event, 60000) != WAIT_OBJECT_0)
        hr = HRESULT_FROM_WIN32(ERROR_TIMEOUT);
    CloseHandle(event);
    if (FAILED(hr)) return hr;
    if (readback) {
        hr = read_metrics(h, metrics);
        if (FAILED(hr)) return hr;
    }
    hr = h.device->GetDeviceRemovedReason();
    if (FAILED(hr)) return hr;
    hr = h.allocator->Reset();
    return FAILED(hr) ? hr : h.list->Reset(h.allocator.Get(), nullptr);
}
