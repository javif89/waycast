#pragma once
#include <filesystem>
#include <string>
#include <vector>
#include <unordered_set>
#include <system_error>
#include <iostream>
#include <sstream>
#include <fstream>

namespace files
{
    namespace fs = std::filesystem;

    std::vector<fs::path> findFilesWithExtension(const std::string path, const std::string ext)
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

    std::string readFile(const std::string &filename)
    {
        std::ifstream in(filename);
        if (!in)
            throw std::runtime_error("Could not open file");

        std::ostringstream ss;
        ss << in.rdbuf(); // dump entire buffer into string
        return ss.str();
    }
}