# NEXRAD Interface Control Documents

This directory contains archived copies of NEXRAD WSR-88D Interface Control Documents
(ICDs) from the NOAA Radar Operations Center (ROC). These are public government
documents used as the authoritative reference for this decoder's implementation.

The ROC's [ICD index page](https://www.roc.noaa.gov/interface-control-documents.php)
only links to the most recent builds, and older documents have been delisted over time.
Many still exist on the server at their original URLs but are no longer discoverable.
This archive preserves them for reference and correctness verification.

## ICD for the RDA/RPG (Document 2620002)

The primary ICD defining the interface between the Radar Data Acquisition (RDA) unit
and the Radar Product Generator (RPG). This is the most relevant document series for
this decoder.

| File | Revision | Build | Date | Notes |
|------|----------|-------|------|-------|
| [2620002B.pdf](rda-rpg/2620002B.pdf) | B | 1.0 | Sep 11, 2001 | Earliest available; "Open Build 1.0" |
| [2620002C.pdf](rda-rpg/2620002C.pdf) | C | 7.0 | Apr 13, 2005 | |
| [2620002D.pdf](rda-rpg/2620002D.pdf) | D | 8.0 | Feb 8, 2006 | |
| [2620002E.pdf](rda-rpg/2620002E.pdf) | E | 9.0 | May 25, 2007 | "Open Build 9.0" |
| [2620002F.pdf](rda-rpg/2620002F.pdf) | F | 10.0 | Mar 25, 2008 | "Open Build 10.0" |
| [2620002G.pdf](rda-rpg/2620002G.pdf) | G | 11.0 | Mar 3, 2009 | "RPG Build 11.0" |
| [2620002H.pdf](rda-rpg/2620002H.pdf) | H | 11.2 | Nov 4, 2009 | "RPG Build 11.2" |
| [2620002J.pdf](rda-rpg/2620002J.pdf) | J | 11.5/12.1 | Jun 7, 2010 | "RDA Build 11.5/RPG Build 12.1"; skips I per standard |
| [2620002M.pdf](rda-rpg/2620002M.pdf) | M | 13.0 | Jun 14, 2012 | "Open Build 13.0" |
| [2620002N.pdf](rda-rpg/2620002N.pdf) | N | 14.0 | Jan 6, 2014 | "RDA Build 14.0" |
| [2620002P.pdf](rda-rpg/2620002P.pdf) | P | 17.0 | Apr 21, 2016 | Skips O per standard |
| [2620002R.pdf](rda-rpg/2620002R.pdf) | R | 18.0 | Feb 28, 2018 | "RDA Build 18.0" |
| [2620002T.pdf](rda-rpg/2620002T.pdf) | T | 19.0 | Mar 3, 2020 | "RDA/RPG Build 19.0" |
| [2620002U.pdf](rda-rpg/2620002U.pdf) | U | 20.0 | Jul 21, 2021 | VolumeDataBlock expanded 40→48 bytes |
| [2620002V.pdf](rda-rpg/2620002V.pdf) | V | 21.0 | Jun 2, 2022 | |
| [2620002W.pdf](rda-rpg/2620002W.pdf) | W | 22.0 | Jun 5, 2023 | |
| [2620002Y.pdf](rda-rpg/2620002Y.pdf) | Y | 23.0 | Jun 25, 2024 | |
| [2620002AA.pdf](rda-rpg/2620002AA.pdf) | AA | 24.0 | Aug 19, 2025 | Latest |

**Missing revisions (not found on any server):** K, L, Q, S, X, Z

**Revision letter convention:** Letters I and O are skipped to avoid confusion with
digits 1 and 0. After Z, revisions continue with AA, AB, etc.

## ICD for Archive II/User (Document 2620010)

Defines the Archive II data format used for storing and distributing base radar data.
This decoder's file reading and decompression logic is based on this document series.

| File | Revision | Build | Date | Notes |
|------|----------|-------|------|-------|
| [2620010A.pdf](archive-ii/2620010A.pdf) | A | 5.0 | Jan 30, 2004 | "Open Build 5.0" |
| [2620010B.pdf](archive-ii/2620010B.pdf) | B | 7.0 | Apr 13, 2005 | "Open Build 7.0" |
| [2620010C.pdf](archive-ii/2620010C.pdf) | C | 8.0 | Feb 8, 2006 | "Open Build 8.0" |
| [2620010E.pdf](archive-ii/2620010E.pdf) | E | 12.0 | May 24, 2010 | "RPG Build 12.0" |
| [2620010G.pdf](archive-ii/2620010G.pdf) | G | 18.0 | Jan 18, 2018 | "RPG Build 18.0" |
| [2620010H.pdf](archive-ii/2620010H.pdf) | H | 19.0 | Mar 3, 2020 | Referenced in codebase |
| [2620010J.pdf](archive-ii/2620010J.pdf) | J | 23.0 | Jun 25, 2024 | |

**Missing revisions:** D, F

## Source URLs

Documents were downloaded from these locations:

- Primary: `https://www.roc.noaa.gov/public-documents/icds/{filename}.pdf`
- Rev P alternate: `https://www.roc.noaa.gov/public-documents/icds/RDA_RPG_2620002P.pdf`

The ROC ICD index page: https://www.roc.noaa.gov/interface-control-documents.php

## Relevance to This Codebase

Key ICD references in the decoder:

- **`nexrad-decode`** primarily implements message formats from ICD 2620002 (RDA/RPG)
  - Digital Radar Data (Message Type 31)
  - RDA Status Data (Message Type 2)
  - Volume Coverage Pattern (Message Type 5) — references Table XI
  - Clutter Filter Map (Message Type 15) — references Table XIV
- **`nexrad-data`** implements the Archive II file format from ICD 2620010
- Build 20.0 introduced a breaking change to the VolumeDataBlock format (40 → 48 bytes)
