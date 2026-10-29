# SPDX-License-Identifier: MPL-2.0

# Independent .NET half decoder and metric implementation; no AMD calls.
param([Parameter(Mandatory)][string]$Stage)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Globalization;
using System.Collections.Generic;
using System.Security.Cryptography;

public static class SharpenReadbackAudit {
    public static IEnumerable<string> Run(string stage) {
        yield return "repeat,scene,setting,SHA256,bit_mismatches,D,C,G,B,G_off,gradient_ratio,criteria";
        foreach (string dir in Directory.GetDirectories(stage, "sharpening-repeat-*.images")) {
            foreach (int scene in new[] {0, 1}) {
                byte[] off = File.ReadAllBytes(Path.Combine(dir, $"scene-{scene}-off_a.red.u16le"));
                double[] baseline = Decode(off);
                double gOff = Gradient(baseline);
                foreach (string setting in new[] {"off_a", "off_b", "zero", "0.35", "1"}) {
                    byte[] bytes = File.ReadAllBytes(Path.Combine(dir, $"scene-{scene}-{setting}.red.u16le"));
                    double[] red = Decode(bytes);
                    double diff = 0, shift = 0;
                    int changed = 0, mismatches = 0, count = 0;
                    for (int y = 32; y < 328; ++y) for (int x = 32; x < 608; ++x) {
                        int i = y * 640 + x;
                        double delta = red[i] - baseline[i];
                        diff += Math.Abs(delta);
                        shift += delta;
                        if (Math.Abs(delta) >= 1.0 / 1024) ++changed;
                        if (bytes[2*i] != off[2*i] || bytes[2*i+1] != off[2*i+1]) ++mismatches;
                        ++count;
                    }
                    double d = diff / count, c = (double)changed / count;
                    double b = Math.Abs(shift / count), g = Gradient(red);
                    bool passed = d >= .001 && c >= .01 && gOff > 0 && g >= 1.01*gOff && b <= .005;
                    string hash = Convert.ToHexString(SHA256.HashData(bytes)).ToLowerInvariant();
                    yield return string.Format(CultureInfo.InvariantCulture,
                        "{0},{1},{2},{3},{4},{5:F10},{6:F10},{7:F10},{8:F10},{9:F10},{10:F10},{11}",
                        Path.GetFileName(dir), scene, setting, hash, mismatches, d, c, g, b, gOff, g/gOff, passed);
                }
            }
        }
    }
    static double[] Decode(byte[] bytes) {
        if (bytes.Length != 640*360*2 || !BitConverter.IsLittleEndian)
            throw new InvalidDataException("Expected tightly packed 640x360 little-endian red half values");
        double[] red = new double[640*360];
        for (int i = 0; i < red.Length; ++i) {
            red[i] = (double)BitConverter.UInt16BitsToHalf(BitConverter.ToUInt16(bytes, 2*i));
            if (double.IsNaN(red[i]) || double.IsInfinity(red[i]))
                throw new InvalidDataException("Non-finite readback");
        }
        return red;
    }
    static double Gradient(double[] red) {
        double sum = 0;
        int count = 0;
        for (int y = 32; y < 328; ++y) for (int x = 32; x < 607; ++x) {
            int i = y*640+x;
            sum += Math.Abs(red[i+1] - red[i]);
            ++count;
        }
        return sum/count;
    }
}
'@
[SharpenReadbackAudit]::Run((Resolve-Path -LiteralPath $Stage).Path)
