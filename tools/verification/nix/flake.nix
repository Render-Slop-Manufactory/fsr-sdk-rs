# SPDX-License-Identifier: MPL-2.0
{
  description = "FSR SDK maintainer verification tools; SDKs and runtimes are prepared separately";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/7a0f122f5090cf4c2ade2a13a0e229d4e19ba71f";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          git gcc rustup cargo-xwin
          llvmPackages.clang-unwrapped llvmPackages.lld
          (python3.withPackages (ps: [ ps.numpy ps.pillow ])) powershell
          umu-launcher-unwrapped osslsigncode vulkan-tools vulkan-validation-layers
        ];
        VERIFICATION_UMU = "${pkgs.umu-launcher-unwrapped}/bin/umu-run";
        VERIFICATION_VULKAN_MANIFEST = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json";
      };
    };
}
