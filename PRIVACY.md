# Privacy

Last updated: May 2, 2026

## Overview

Lap is designed as a local-first photo manager. Your photo library is processed on your device, and your files remain under your control.

This document explains what data Lap accesses, what data Lap does not collect by default, and when network access may occur.

## What Lap Accesses

Lap may access the following data on your device in order to provide its features:

- Photos, videos, and folders that you choose to add to a library
- File metadata such as filenames, paths, timestamps, size, format, EXIF data, ratings, tags, comments, and rotation state
- Generated local app data such as thumbnails, previews, indexes, search data, embeddings, face clustering data, video indexing data, and other library-related cache or database records

Lap uses this data to support browsing, search, deduplication, tagging, ratings, file type filtering, video support, face clustering, and other library management features.

## Local Processing

Lap is intended to process your library locally on your device.

By default:

- Your photos and videos are not uploaded to a Lap cloud service
- AI search, smart tags, face detection, face clustering, thumbnail generation, RAW previews, and video indexing are processed locally
- Lap does not include advertising trackers
- Lap does not track daily active usage, session counts, or feature usage patterns
- The frontend does not record user behavior, clicks, or navigation events

## Network Access

Lap may access the network in limited cases where the feature requires it. Based on the current implementation, this may include:

- Checking for application updates
- Downloading application updates from GitHub releases when you choose to install an update
- Opening external links such as the project website or GitHub repository in your browser
- Fetching map tiles from OpenStreetMap tile servers when viewing a photo's GPS location on the map

## Anonymous Usage Statistics

Lap includes Aptabase, a privacy-first analytics service, in its backend. It is enabled by default in release builds to help the maintainer understand basic adoption and stability.

Aptabase runs only on the Rust backend. The frontend does not record or send any user behavior events — no clicks, no navigation tracking, no feature usage analytics.

When enabled, Lap sends exactly two app-lifecycle events:

- App started
- App exited

Each event carries only the Lap version, device platform, and operating system. No coarse region or IP-derived location is stored or surfaced by the analytics service.

These events are strictly anonymous. They contain no user identifiers, session IDs, or device fingerprints. Aptabase does not use cookies or tracking pixels, and events are not correlated across sessions.

**Lap does not and will never send** your photos, videos, folder paths, filenames, search queries, tags, ratings, comments, EXIF data, embeddings, face clusters, thumbnails, previews, database contents, or any other library data.

## Data Storage

Lap stores application data locally on your device. This may include:

- App settings
- Local databases for library records, generated thumbnails and previews, search indexes, embeddings, face clustering data, video indexing data, tags, ratings, comments, and related cache data

This local data is used to provide the app's functionality and improve performance on your device.

## Third-Party Services

Lap does not provide its own cloud storage service.

When you use update-related features, release assets may be fetched from GitHub. Those requests are subject to GitHub's terms and privacy practices.

When anonymous analytics is enabled, event delivery is handled by Aptabase. Those requests are subject to Aptabase's terms and privacy practices.

## Changes to This Document

This document may be updated as the app evolves. Privacy-related changes should be reflected in the repository and user-facing documentation.

## Contact

For privacy-related questions or concerns, please open an issue in the project repository:

[https://github.com/julyx10/lap](https://github.com/julyx10/lap)
