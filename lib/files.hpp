#pragma once
#include <filesystem>
#include <string>
#include <vector>
#include <unordered_set>
#include <system_error>
#include <iostream>
#include <sstream>
#include <fstream>
#include <functional>

namespace files
{
    namespace fs = std::filesystem;

    inline std::vector<fs::path> findFilesWithExtension(const std::string path, const std::string ext)
    {
        std::vector<fs::path> out;
        std::unordered_set<std::string> seen; // canonicalized paths to dedupe
        std::error_code ec;
        fs::path p(path);

        if (!fs::exists(p, ec) || !fs::is_directory(p, ec))
            return out;

        for (const auto &entry : fs::directory_iterator(p, ec))
        {
            if (ec)
            {
                ec.clear();
                continue;
            }

            const auto &filePath = entry.path();
            if (filePath.extension() == ext && fs::is_regular_file(filePath, ec))
            {
                out.push_back(filePath);
            }
        }

        return out;
    }

    inline std::vector<fs::path> findAllFiles(const std::string& path, int max_depth = -1)
    {
        std::vector<fs::path> out;
        std::error_code ec;
        fs::path p(path);

        if (!fs::exists(p, ec) || !fs::is_directory(p, ec))
            return out;

        // Use recursive_directory_iterator with depth control
        auto options = fs::directory_options::skip_permission_denied;
        
        try {
            if (max_depth < 0) {
                // No depth limit - use unlimited recursion
                for (const auto& entry : fs::recursive_directory_iterator(p, options, ec)) {
                    if (ec) {
                        ec.clear();
                        continue;
                    }
                    
                    if (entry.is_regular_file(ec) && !ec) {
                        out.push_back(entry.path());
                    }
                    ec.clear();
                }
            } else {
                // Limited depth recursion
                std::function<void(const fs::path&, int)> recurse = [&](const fs::path& dir_path, int current_depth) {
                    if (current_depth > max_depth) return;
                    
                    for (const auto& entry : fs::directory_iterator(dir_path, options, ec)) {
                        if (ec) {
                            ec.clear();
                            continue;
                        }
                        
                        if (entry.is_regular_file(ec) && !ec) {
                            out.push_back(entry.path());
                        } else if (entry.is_directory(ec) && !ec && current_depth < max_depth) {
                            recurse(entry.path(), current_depth + 1);
                        }
                        ec.clear();
                    }
                };
                
                recurse(p, 0);
            }
        } catch (const fs::filesystem_error&) {
            // Handle any remaining filesystem errors silently
        }

        return out;
    }

    inline std::string readFile(const std::string &filename)
    {
        std::ifstream in(filename);
        if (!in)
            throw std::runtime_error("Could not open file");

        std::ostringstream ss;
        ss << in.rdbuf(); // dump entire buffer into string
        return ss.str();
    }
}