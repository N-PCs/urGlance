#include "urglance/src/preview.h"
#include "urglance/src/main.rs.h"

#include <iostream>
#include <fstream>
#include <sstream>
#include <filesystem>
#include <chrono>
#include <algorithm>
#include <cctype>
#include <cmath>

namespace urglance {

// Format bytes into human-readable size
std::string format_size(uint64_t bytes) {
    const char* suffixes[] = { "Bytes", "KB", "MB", "GB", "TB" };
    int i = 0;
    double dbl_bytes = static_cast<double>(bytes);
    while (dbl_bytes >= 1024.0 && i < 4) {
        dbl_bytes /= 1024.0;
        i++;
    }
    std::ostringstream out;
    out << std::fixed;
    if (i == 0) {
        out << static_cast<uint64_t>(dbl_bytes) << " " << suffixes[i];
    } else {
        out.precision(2);
        out << dbl_bytes << " " << suffixes[i];
    }
    return out.str();
}

// Format filesystem permissions into rwxrwxrwx format
std::string format_permissions(std::filesystem::perms p) {
    std::string r;
    using std::filesystem::perms;
    r += ((p & perms::owner_read) != perms::none ? "r" : "-");
    r += ((p & perms::owner_write) != perms::none ? "w" : "-");
    r += ((p & perms::owner_exec) != perms::none ? "x" : "-");
    r += ((p & perms::group_read) != perms::none ? "r" : "-");
    r += ((p & perms::group_write) != perms::none ? "w" : "-");
    r += ((p & perms::group_exec) != perms::none ? "x" : "-");
    r += ((p & perms::others_read) != perms::none ? "r" : "-");
    r += ((p & perms::others_write) != perms::none ? "w" : "-");
    r += ((p & perms::others_exec) != perms::none ? "x" : "-");
    return r;
}

// Parse PNG dimensions
bool parse_png_dimensions(const std::filesystem::path& path, uint32_t& width, uint32_t& height) {
    std::ifstream file(path, std::ios::binary);
    if (!file) return false;

    char buffer[24];
    if (!file.read(buffer, 24)) return false;

    // Check PNG signature: 89 50 4E 47 0D 0A 1A 0A
    if (static_cast<unsigned char>(buffer[0]) == 0x89 &&
        buffer[1] == 'P' && buffer[2] == 'N' && buffer[3] == 'G' &&
        buffer[4] == '\r' && buffer[5] == '\n' && buffer[6] == 0x1a && buffer[7] == '\n') {
        
        // IHDR chunk starts at byte 12 (length 13, type "IHDR")
        // Width is at bytes 16-19 (big-endian)
        // Height is at bytes 20-23 (big-endian)
        width = (static_cast<unsigned char>(buffer[16]) << 24) |
                (static_cast<unsigned char>(buffer[17]) << 16) |
                (static_cast<unsigned char>(buffer[18]) << 8)  |
                (static_cast<unsigned char>(buffer[19]));
        height = (static_cast<unsigned char>(buffer[20]) << 24) |
                 (static_cast<unsigned char>(buffer[21]) << 16) |
                 (static_cast<unsigned char>(buffer[22]) << 8)  |
                 (static_cast<unsigned char>(buffer[23]));
        return true;
    }
    return false;
}

// Parse BMP dimensions
bool parse_bmp_dimensions(const std::filesystem::path& path, uint32_t& width, uint32_t& height) {
    std::ifstream file(path, std::ios::binary);
    if (!file) return false;

    char buffer[26];
    if (!file.read(buffer, 26)) return false;

    // Check BMP signature: "BM"
    if (buffer[0] == 'B' && buffer[1] == 'M') {
        // DIB header size is at offset 14 (4 bytes, little-endian)
        uint32_t dib_size = (static_cast<unsigned char>(buffer[14])) |
                            (static_cast<unsigned char>(buffer[15]) << 8) |
                            (static_cast<unsigned char>(buffer[16]) << 16) |
                            (static_cast<unsigned char>(buffer[17]) << 24);
        
        if (dib_size == 12) { // BITMAPCOREHEADER
            width = (static_cast<unsigned char>(buffer[18])) | (static_cast<unsigned char>(buffer[19]) << 8);
            height = (static_cast<unsigned char>(buffer[20])) | (static_cast<unsigned char>(buffer[21]) << 8);
        } else { // BITMAPINFOHEADER and newer variants (width/height at same offsets)
            int32_t w = (static_cast<unsigned char>(buffer[18])) |
                        (static_cast<unsigned char>(buffer[19]) << 8) |
                        (static_cast<unsigned char>(buffer[20]) << 16) |
                        (static_cast<unsigned char>(buffer[21]) << 24);
            int32_t h = (static_cast<unsigned char>(buffer[22])) |
                        (static_cast<unsigned char>(buffer[23]) << 8) |
                        (static_cast<unsigned char>(buffer[24]) << 16) |
                        (static_cast<unsigned char>(buffer[25]) << 24);
            width = w;
            height = std::abs(h); // Height can be negative for top-down BMPs
        }
        return true;
    }
    return false;
}

// Parse JPEG dimensions
bool parse_jpeg_dimensions(const std::filesystem::path& path, uint32_t& width, uint32_t& height) {
    std::ifstream file(path, std::ios::binary);
    if (!file) return false;

    unsigned char marker[2];
    if (!file.read(reinterpret_cast<char*>(marker), 2)) return false;
    if (marker[0] != 0xFF || marker[1] != 0xD8) return false; // SOI marker check

    while (file.read(reinterpret_cast<char*>(marker), 2)) {
        if (marker[0] != 0xFF) {
            // Find next 0xFF boundary
            while (marker[0] != 0xFF && file.read(reinterpret_cast<char*>(&marker[0]), 1)) {}
            if (marker[0] != 0xFF) break;
            if (!file.read(reinterpret_cast<char*>(&marker[1]), 1)) break;
        }

        // Skip duplicate 0xFF bytes
        while (marker[1] == 0xFF) {
            if (!file.read(reinterpret_cast<char*>(&marker[1]), 1)) break;
        }

        unsigned char m_byte = marker[1];

        // Stop scanning if SOS or EOI reached
        if (m_byte == 0xDA || m_byte == 0xD9) {
            break;
        }

        // Skip markers without length
        if (m_byte == 0xD8 || (m_byte >= 0xD0 && m_byte <= 0xD7)) {
            continue;
        }

        // Read 2-byte marker payload length
        unsigned char len_bytes[2];
        if (!file.read(reinterpret_cast<char*>(len_bytes), 2)) break;
        uint16_t length = (len_bytes[0] << 8) | len_bytes[1];
        if (length < 2) break;

        // Check if Start Of Frame (SOF) marker
        // SOF markers: 0xC0-0xC3, 0xC5-0xC7, 0xC9-0xCB, 0xCD-0xCF
        if ((m_byte >= 0xC0 && m_byte <= 0xCF) && m_byte != 0xC4 && m_byte != 0xC8 && m_byte != 0xCC) {
            unsigned char sof_data[5];
            if (file.read(reinterpret_cast<char*>(sof_data), 5)) {
                height = (sof_data[1] << 8) | sof_data[2];
                width = (sof_data[3] << 8) | sof_data[4];
                return true;
            }
            break;
        }

        // Skip payload
        file.seekg(length - 2, std::ios::cur);
    }
    return false;
}

// Core FFI parsing engine implementation
PreviewData extract_file_preview(rust::Str file_path) {
    PreviewData data;
    data.success = false;

    std::string path_str(file_path);
    std::filesystem::path path(path_str);

    // Validate path existence
    if (!std::filesystem::exists(path)) {
        data.file_type = "Unknown / Non-existent";
        data.content_snippet = "[File not found on disk]";
        data.metadata_summary = "N/A";
        return data;
    }

    // Handle Directories
    if (std::filesystem::is_directory(path)) {
        data.file_type = "Directory";
        data.content_snippet = "[Directory Listing Preview Not Displayed]";
        try {
            uint64_t entry_count = 0;
            for (const auto& entry : std::filesystem::directory_iterator(path)) {
                (void)entry;
                entry_count++;
            }
            std::ostringstream meta;
            meta << "Items: " << entry_count;
            data.metadata_summary = meta.str();
            data.success = true;
        } catch (...) {
            data.metadata_summary = "Permissions Error (Could not scan directory)";
        }
        return data;
    }

    // Default metadata placeholders
    std::string size_str = "Unknown Size";
    std::string time_str = "Unknown Date";
    std::string perms_str = "---------";

    // Resilient try/catch block to extract standard metadata
    try {
        uint64_t bytes = std::filesystem::file_size(path);
        size_str = format_size(bytes);

        auto ftime = std::filesystem::last_write_time(path);
        std::time_t c_time = 0;
        #if defined(__cpp_lib_chrono) && __cpp_lib_chrono >= 201907L
            c_time = std::chrono::system_clock::to_time_t(std::chrono::file_clock::to_sys(ftime));
        #else
            auto duration = ftime.time_since_epoch();
            c_time = std::chrono::duration_cast<std::chrono::seconds>(duration).count();
            #if defined(_MSC_VER)
                c_time -= 11644473600LL;
            #endif
        #endif

        std::tm time_info;
        #if defined(_MSC_VER)
            localtime_s(&time_info, &c_time);
        #else
            localtime_r(&c_time, &time_info);
        #endif
        char time_buf[64];
        std::strftime(time_buf, sizeof(time_buf), "%Y-%m-%d %H:%M:%S", &time_info);
        time_str = time_buf;

        perms_str = format_permissions(std::filesystem::status(path).permissions());
    } catch (...) {
        // Fallback gracefully if standard filesystem checks fail
    }

    std::string ext = path.extension().string();
    std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char c) { return std::tolower(c); });

    // 1. Plain Text and Code Parsers
    if (ext == ".txt" || ext == ".py" || ext == ".json" || ext == ".csv" || ext == ".cpp" || ext == ".h" || ext == ".rs" || ext == ".toml" || ext == ".md" || ext == ".html" || ext == ".css" || ext == ".js") {
        data.file_type = "Text / Code File (" + ext + ")";
        try {
            std::ifstream file(path);
            if (file) {
                std::string line;
                std::string snippet;
                size_t lines_read = 0;
                size_t total_chars = 0;
                const size_t MAX_CHARS = 4096;
                const size_t MAX_LINES = 15;

                while (lines_read < MAX_LINES && total_chars < MAX_CHARS && std::getline(file, line)) {
                    if (total_chars + line.length() + 1 > MAX_CHARS) {
                        size_t allowed = MAX_CHARS - total_chars;
                        snippet += line.substr(0, allowed) + "...";
                        break;
                    }
                    snippet += line + "\n";
                    total_chars += line.length() + 1;
                    lines_read++;
                }
                data.content_snippet = snippet;
                data.success = true;
            } else {
                data.content_snippet = "[Could not open file handle]";
            }
        } catch (...) {
            data.content_snippet = "[Error reading file buffer]";
        }

        std::ostringstream meta;
        meta << "Size: " << size_str << " | Modified: " << time_str << " | Permissions: " << perms_str;
        data.metadata_summary = meta.str();
    }
    // 2. Fast Image Header Parsers
    else if (ext == ".png" || ext == ".jpg" || ext == ".jpeg" || ext == ".bmp") {
        data.file_type = "Image File (" + ext + ")";
        uint32_t width = 0;
        uint32_t height = 0;
        bool parse_success = false;

        try {
            if (ext == ".png") {
                parse_success = parse_png_dimensions(path, width, height);
            } else if (ext == ".jpg" || ext == ".jpeg") {
                parse_success = parse_jpeg_dimensions(path, width, height);
            } else if (ext == ".bmp") {
                parse_success = parse_bmp_dimensions(path, width, height);
            }
        } catch (...) {
            parse_success = false;
        }

        std::ostringstream meta;
        meta << "Size: " << size_str << " | ";
        if (parse_success) {
            meta << "Dimensions: " << width << "x" << height << " | ";
        } else {
            meta << "Dimensions: Unknown (Header Corrupted) | ";
        }
        meta << "Modified: " << time_str << " | Permissions: " << perms_str;
        data.metadata_summary = meta.str();
        data.content_snippet = "[Binary Data - Image Preview Not Available]";
        data.success = true;
    }
    // 3. Fallback for Unsupported/Binary Formats
    else {
        data.file_type = "Binary / Generic (" + ext + ")";
        data.content_snippet = "[Binary Data - Preview Not Available]";
        std::ostringstream meta;
        meta << "Size: " << size_str << " | Modified: " << time_str << " | Permissions: " << perms_str;
        data.metadata_summary = meta.str();
        data.success = true;
    }

    return data;
}

} // namespace urglance
