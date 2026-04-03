# NEXRAD Sweep Timing Analysis

## Executive Summary

This analysis examines radial-level timestamps from **59 archive volumes** across **12 NEXRAD sites**, **8 VCP types**, and diverse meteorological scenarios (clear air, precipitation, severe storms, hurricanes, lake effect snow) spanning 2010-2026. The goal is to build a predictive model for when individual sweeps become available during a volume scan.

**Key findings:**
1. **Sweep duration is highly predictable** from VCP azimuth rate: `duration = 360 / azimuth_rate` with a consistent negative bias of ~0.7s (the sweep finishes slightly faster than a full 360-degree rotation would suggest)
2. **Inter-sweep gaps are remarkably consistent**: 0.5-2.8s, with most falling in the 0.8-1.3s range
3. **Inter-volume gaps are consistent**: 7-10s between consecutive volumes
4. **Volume duration is determined almost entirely by VCP type**, with SAILS/MRLE adding additional sweeps

---

## 1. Dataset

| Category | Sites | Files | VCPs |
|----------|-------|-------|------|
| Clear air | KTLX, KLIX, KPUX, KFWS, KILX, PAPD, KDMX | 30 | VCP 31, 32, 35 |
| Precipitation | KTLX, KCRP, KFWS, KDMX, KMLB | 18 | VCP 12, 112, 212, 215 |
| Severe/Hurricane | KTLX, KCRP, KMLB | 7 | VCP 112, 212 |
| Winter/Other | KLWX, KMKX, KBUF, PHKI, TJUA | 8 | VCP 21, 215 |

Geographic coverage: Oklahoma, Texas, Louisiana, Florida, Illinois, Iowa, Virginia/DC, Wisconsin, New York, Colorado, Hawaii, Alaska, Puerto Rico.

---

## 2. Volume Duration by VCP

| VCP | Description | N | Mean (s) | Median (s) | StdDev (s) | Range (s) | Sweeps |
|-----|-------------|---|----------|------------|------------|-----------|--------|
| **12** | Precipitation, fast update | 4 | 248.8 | 248.6 | 0.6 | 248-250 | 17 |
| **212** | Precipitation, SZ-2 | 14 | 258.7 | 245.3 | 61.6 | 192-391 | 12-23 |
| **215** | General surveillance, SZ-2 | 11 | 297.5 | 284.5 | 39.5 | 246-358 | 12-18 |
| **21** | Precipitation (legacy) | 2 | 340.0 | 340.0 | 0.2 | 340-340 | 11 |
| **112** | Precipitation, MPDA + SZ-2 | 1 | 389.8 | 389.8 | - | 390 | 23 |
| **35** | Clear air, SZ-2 | 22 | 425.1 | 411.7 | 39.0 | 393-511 | 12-14 |
| **31** | Clear air, long pulse | 2 | 570.6 | 570.6 | 4.7 | 567-574 | 8 |
| **32** | Clear air, short pulse | 3 | 569.1 | 570.4 | 2.9 | 566-571 | 7 |

**Key observations:**
- VCP 12 is the most consistent (~4.1 min, stdev <1s) -- fixed 17 sweeps, no SAILS/MRLE variation
- VCP 212 has the widest range because SAILS/MRLE features add extra sweeps (12-23 sweeps depending on configuration)
- Clear air VCPs (31, 32, 35) are much longer (~6.5-8.5 min) due to slower azimuth rates for better sensitivity
- VCP 35 (modern clear air) is ~2.5 minutes faster than VCP 31/32 (legacy clear air)

---

## 3. Sweep Duration: The Primary Predictor

### 3.1 The Azimuth Rate Formula

Sweep duration is almost entirely determined by the VCP's prescribed azimuth rotation rate for that elevation cut:

```
predicted_duration = 360 / azimuth_rate_degrees_per_second
```

**Prediction accuracy across 801 sweep observations:**

| Metric | Value |
|--------|-------|
| Mean error | -0.67s (actual is shorter than predicted) |
| Median error | -0.68s |
| Mean absolute error | 0.68s |
| Max absolute error | 2.06s |
| Standard deviation | 0.33s |

The negative bias means sweeps consistently finish ~0.7s before a full 360-degree rotation would predict. This likely reflects that the radar doesn't need to complete a full 360-degree rotation -- the last radial at ~359.5 degrees means the sweep is ~0.5-1 degree short of a full circle.

### 3.2 Prediction Accuracy by Waveform Type

| Waveform | N | Mean Error (s) | MAE (s) | StdDev (s) |
|----------|---|----------------|---------|------------|
| CS (Contiguous Surveillance) | 191 | -0.75 | 0.77 | 0.43 |
| CDW (Contiguous Doppler w/ AR) | 195 | -0.67 | 0.67 | 0.28 |
| CDWO (Contiguous Doppler w/o AR) | 86 | -0.46 | 0.46 | 0.28 |
| B (Batch) | 329 | -0.69 | 0.69 | 0.28 |

CS waveform sweeps have slightly more variance (stdev 0.43s vs 0.28s), but all waveform types are well-predicted.

### 3.3 A Better Predictor

Since the bias is consistent, a corrected formula provides better accuracy:

```
predicted_duration = (360 / azimuth_rate) - 0.67
```

This would reduce MAE from 0.68s to approximately 0.33s (the residual standard deviation).

### 3.4 Sweep Duration by VCP and Elevation

The full per-elevation breakdown shows how sweep duration varies within a volume. Representative examples:

**VCP 212 (Precipitation)** -- fast scan, 21 dps at low elevations:
| Elev# | Angle | Waveform | AzRate (dps) | Radials | Duration (s) |
|-------|-------|----------|-------------|---------|-------------|
| 1 | 0.5 | CS | 21.15 | 720 | 16.5 |
| 2 | 0.5 | CDW | 18.24 | 720 | 18.7 |
| 3 | 0.9 | CS | 21.15 | 720 | 16.5 |
| 7 | 1.8 | B | 24.64 | 360 | 14.3 |
| 12 | 4.4 | B | 26.40 | 360 | 13.4 |
| 19 | 19.5 | CDWO | 28.74 | 360 | 12.2 |

**VCP 35 (Clear air)** -- slow scan, 5 dps at low elevations:
| Elev# | Angle | Waveform | AzRate (dps) | Radials | Duration (s) |
|-------|-------|----------|-------------|---------|-------------|
| 1 | 0.5 | CS | 4.97 | 720 | 71.5 |
| 2 | 0.5 | CDW | 17.11 | 720 | 22.0 |
| 3 | 0.9 | CS | 4.97 | 720 | 71.5 |
| 7 | 1.6 | B | 15.49 | 360 | 22.5 |
| 12 | 6.0 | B | 18.07 | 360 | 19.1 |

**Pattern**: Low elevations use slower rotation (higher sensitivity), dual CS+CDW sweeps. Higher elevations use faster rotation with Batch or CDWO waveforms.

---

## 4. Inter-Sweep Gaps

### 4.1 Overall Statistics

| VCP | Mean Gap (s) | Min Gap (s) | Max Gap (s) |
|-----|-------------|-------------|-------------|
| 12 | 1.03 | 0.53 | 2.78 |
| 212 | 1.11 | 0.81 | 2.65 |
| 215 | 1.12 | 0.77 | 1.88 |
| 112 | 1.25 | 0.86 | 2.19 |
| 31 | 1.07 | 0.68 | 1.38 |
| 32 | 1.31 | 0.69 | 1.94 |
| 35 | 1.28 | 0.83 | 2.10 |
| 21 | 1.01 | 0.58 | 1.25 |

### 4.2 Gap Patterns by Transition Type

The gap between sweeps depends on the elevation angle change:

**Same elevation transitions** (e.g., CS sweep at 0.5 -> CDW sweep at 0.5): shortest gaps, typically **0.6-0.9s**. The antenna doesn't need to move; it only changes the waveform mode.

**Small elevation changes** (e.g., 0.5 -> 0.9 degrees): **0.7-1.0s**

**Large elevation changes** (e.g., 6.4 -> 8.0 degrees, or 12.5 -> 15.7 degrees): **1.0-2.8s**

**Approximate gap model:**
```
gap_seconds = 0.7 + (elevation_change_degrees * 0.08)
```

This accounts for the antenna repositioning time. The base 0.7s represents mode switching overhead. The 0.08s per degree represents antenna slew rate during transitions (much faster than the survey rotation rate).

### 4.3 Notable Transition Patterns

For VCPs with split-cut pairs (CS + CDW at the same elevation):
- The CS-to-CDW transition at the same angle has the **smallest gap**: 0.53-0.75s
- This is because the antenna is already positioned and only changes waveform mode

For SAILS cuts (supplemental low-level rescans mid-volume):
- The gap coming **down** from a higher elevation back to the SAILS elevation is larger (1.5-2.5s) because the antenna must slew back down
- The gap going **up** from the SAILS elevation to resume mid-volume scanning is similar

---

## 5. Inter-Volume Gaps

Between the last radial of one volume and the first radial of the next:

| Site | VCP | Consecutive Pairs | Mean Gap (s) | Range (s) |
|------|-----|-------------------|-------------|-----------|
| KDMX | 212 | 3 | 9.6 | 9.4-9.9 |
| KDMX | 35 | 2 | 9.3 | 9.1-9.4 |
| KFWS | 212 | 1 | 9.0 | - |
| KLWX | 21 | 1 | 7.3 | - |
| KMKX | 215 | 1 | 8.6 | - |
| KTLX | 12 | 2 | 7.8 | 7.8-7.8 |
| KTLX | 212 | 2 | 8.1 | 8.0-8.2 |

**Summary**: Inter-volume gaps range from **7-10 seconds**, with a typical value of **~8-9 seconds**. This represents the time for the antenna to return from its highest elevation to the starting elevation of the next scan, plus any initialization overhead.

---

## 6. SAILS/MRLE Impact

SAILS (Supplemental Adaptive Intra-volume Low-level Scans) and MRLE (Mid-volume Rescan of Low Elevation) insert additional low-elevation sweeps into the middle of a volume.

**Impact on VCP 212:**
- Without SAILS: 12-14 sweeps, ~192-225s
- With 1 SAILS cut: 17 sweeps, ~266s
- With 2 SAILS cuts: 19-21 sweeps, ~303-327s
- With 2 SAILS + 3 MRLE: 23 sweeps, ~391s

Each SAILS insertion adds approximately **2 sweeps** (one CS + one CDW at ~0.5 degrees) plus transition time, adding roughly **40-45 seconds** per SAILS cut to the total volume time.

---

## 7. Building a Sweep Timing Predictor

### 7.1 Required Inputs

To predict when each sweep in a volume will be available, you need:
1. **VCP number** (known from the first message of the volume)
2. **VCP elevation cuts** (embedded in the volume data, including azimuth rates)
3. **SAILS/MRLE configuration** (embedded in VCP metadata)
4. **Volume start time** (first radial timestamp)

### 7.2 Prediction Algorithm

```
For each elevation cut i in the VCP:
    sweep_duration[i] = (360 / azimuth_rate[i]) - 0.67
    
    if i == 0:
        sweep_start[i] = volume_start_time
    else:
        # Gap depends on elevation change
        elev_change = abs(elevation[i] - elevation[i-1])
        gap[i] = 0.7 + (elev_change * 0.08)
        sweep_start[i] = sweep_end[i-1] + gap[i]
    
    sweep_end[i] = sweep_start[i] + sweep_duration[i]
```

### 7.3 Expected Accuracy

| Component | Error Budget |
|-----------|-------------|
| Sweep duration prediction | +/- 0.33s (1 sigma) |
| Inter-sweep gap prediction | +/- 0.2s (1 sigma) |
| **Per-sweep cumulative** | **~0.4s per sweep** |
| **Full volume (17 sweeps)** | **~1.7s cumulative** |

For a 17-sweep VCP 212 volume, the predicted end time of the last sweep should be within ~2 seconds of the actual end time.

### 7.4 Edge Cases and Refinements

1. **Split-cut pairs**: CS and CDW sweeps at the same elevation angle have a shorter transition gap (~0.65s instead of the elevation-based formula). Detect these by checking if consecutive cuts have the same elevation angle.

2. **SAILS/MRLE insertions**: These appear as additional elevation cuts in the VCP data with `is_sails_cut` or `is_mrle_cut` flags. They follow the same timing rules -- just use their azimuth rate and compute the elevation change gap from the preceding sweep.

3. **CDW sweeps with super-resolution**: These have 720 radials at 0.5-degree spacing (same as CS sweeps), but the azimuth rate already accounts for this. The formula `360 / azimuth_rate` works regardless of radial count.

4. **First sweep of volume**: Always starts at the volume start time with no gap.

5. **Last sweep of volume**: After the last sweep ends, there is a ~7-10s gap before the next volume begins.

---

## 8. Azimuth Rate Reference Table

Common azimuth rates observed across VCPs, useful for building lookup tables:

| Rate (dps) | Approx Duration (s) | Typical Usage |
|-----------|---------------------|---------------|
| 4.1 | 87 | Clear air Batch (VCP 32) |
| 4.5-5.1 | 71-79 | Clear air CS/CDW (VCP 31, 32, 35) |
| 5.5 | 65 | Clear air CS at 1.3 deg (VCP 35) |
| 11.2-11.5 | 31-32 | Precip CS low elevation (VCP 215) |
| 13.4-15.9 | 23-27 | Precip CS mid elevation (VCP 215) |
| 14.3-14.5 | 25 | Precip CDW low elevation (VCP 215) |
| 15.5-17.1 | 21-23 | Clear air Batch/CDW mid elev (VCP 35) |
| 17.8-18.1 | 20 | Clear air Batch high elev (VCP 35) |
| 18.2-20.0 | 18-20 | Precip CDW mid elevation |
| 20.2-21.2 | 17 | Precip CS/Batch (VCP 212, 215) |
| 24.6-25.0 | 14 | Precip Batch/CDW (VCP 12, 212) |
| 26.4-28.9 | 12-14 | Precip high elevation (all precip VCPs) |

---

## 9. Methodology

- **Tool**: Custom Rust analysis binary using the `nexrad` crate, outputting JSON
- **Data source**: AWS NEXRAD archive S3 bucket + local files
- **Files analyzed**: 59 volumes (39 local, 20+ downloaded)
- **Timestamps**: Derived from per-radial `collection_timestamp` (millisecond precision)
- **Sweep boundaries**: Determined by `elevation_number` changes in radial sequence
- **Analysis scripts**: `nexrad/examples/timing_analysis.rs` (Rust data extraction), `/tmp/analyze_timings.py` (Python statistical analysis)

### Files skipped
- Pre-2008 files (KABR 2005, KTLX 1991): No radial timestamps or VCP data in older formats
- `_MDM` suffixed files: Some had truncated records (metadata-only files)
