#include <string>
#include <filesystem>
#include <vector>
#include <algorithm>
#include <type_traits>
#include <immintrin.h> // AVX/AVX2 intrinsics

namespace fuzzy
{
    struct FuzzyMatch
    {
        std::string text;
        int score;
        std::vector<size_t> matchPositions;

        bool operator<(const FuzzyMatch &other) const
        {
            return score > other.score; // Higher score = better match
        }
    };

    class FuzzyFinder
    {
    public:
        // Find method for string candidates
        std::vector<FuzzyMatch> find(const std::vector<std::string> &candidates,
                                     const std::string &query,
                                     int limit = 10)
        {
            return findInternal(candidates, query, limit,
                                [](const std::string &candidate) -> std::string
                                {
                                    return candidate;
                                });
        }

        // Find method for filesystem path candidates
        std::vector<FuzzyMatch> find(const std::vector<std::filesystem::path> &candidates,
                                     const std::string &query,
                                     int limit = 10)
        {
            return findInternal(candidates, query, limit,
                                [](const std::filesystem::path &candidate) -> std::string
                                {
                                    // For now, just use filename for matching
                                    // In the future, could implement path-specific scoring
                                    return candidate.filename().string();
                                });
        }

    private:
        // Generic internal find method using a converter function
        template <typename T, typename Converter>
        std::vector<FuzzyMatch> findInternal(const std::vector<T> &candidates,
                                             const std::string &query,
                                             int limit,
                                             Converter converter)
        {
            if (query.empty())
            {
                return {};
            }

            std::vector<FuzzyMatch> matches;
            matches.reserve(candidates.size());

            for (const auto &candidate : candidates)
            {
                std::string searchText = converter(candidate);
                auto match = calculateMatch(searchText, query);
                if (match.score > 0)
                {
                    // Store the original candidate as text for display
                    if constexpr (std::is_same_v<T, std::filesystem::path>)
                    {
                        match.text = candidate.string(); // Full path for display
                    }
                    else
                    {
                        match.text = searchText; // Use converted text
                    }
                    matches.push_back(std::move(match));
                }
            }

            // Sort by score (highest first)
            std::sort(matches.begin(), matches.end());

            // Limit results
            if (limit > 0 && matches.size() > static_cast<size_t>(limit))
            {
                matches.resize(limit);
            }

            return matches;
        }

        // Check if AVX2 is available at runtime
        bool hasAVX2() const
        {
            return __builtin_cpu_supports("avx2");
        }

        // AVX2-accelerated character search - finds next occurrence of query_char in text
        size_t findNextCharAVX2(const std::string &text, char query_char, size_t start_pos) const
        {
            const size_t text_len = text.length();
            if (start_pos >= text_len)
                return std::string::npos;

            const char *text_ptr = text.c_str() + start_pos;
            const size_t remaining = text_len - start_pos;

            // Process 32 characters at a time with AVX2
            const size_t avx_chunks = remaining / 32;
            const __m256i query_vec = _mm256_set1_epi8(query_char);

            for (size_t chunk = 0; chunk < avx_chunks; ++chunk)
            {
                const __m256i text_vec = _mm256_loadu_si256(reinterpret_cast<const __m256i *>(text_ptr + chunk * 32));
                const __m256i cmp_result = _mm256_cmpeq_epi8(query_vec, text_vec);
                const uint32_t match_mask = _mm256_movemask_epi8(cmp_result);

                if (match_mask != 0)
                {
                    // Found a match - find the first set bit
                    const int first_match = __builtin_ctz(match_mask);
                    return start_pos + chunk * 32 + first_match;
                }
            }

            // Handle remaining characters with scalar code
            const size_t remaining_start = start_pos + avx_chunks * 32;
            for (size_t i = remaining_start; i < text_len; ++i)
            {
                if (text[i] == query_char)
                {
                    return i;
                }
            }

            return std::string::npos;
        }

        FuzzyMatch calculateMatch(const std::string &text, const std::string &query)
        {
            // Choose algorithm based on AVX2 availability and string length
            if (hasAVX2() && text.length() > 64 && query.length() > 2)
            {
                return calculateMatchAVX2(text, query);
            }
            else
            {
                return calculateMatchScalar(text, query);
            }
        }

        // AVX2-accelerated fuzzy matching
        FuzzyMatch calculateMatchAVX2(const std::string &text, const std::string &query)
        {
            FuzzyMatch match;
            match.text = text;
            match.score = 0;

            if (query.empty() || text.empty())
            {
                return match;
            }

            std::string lowerText = toLower(text);
            std::string lowerQuery = toLower(query);

            size_t textIdx = 0;
            size_t queryIdx = 0;
            int consecutiveBonus = 0;
            bool inConsecutiveMatch = false;

            // Use AVX2 to accelerate character searching
            while (queryIdx < lowerQuery.length())
            {
                char queryChar = lowerQuery[queryIdx];

                // Use AVX2 to find next occurrence of query character
                size_t foundPos = findNextCharAVX2(lowerText, queryChar, textIdx);

                if (foundPos == std::string::npos)
                {
                    break; // Character not found
                }

                textIdx = foundPos;
                match.matchPositions.push_back(textIdx);

                // Calculate score (same logic as before)
                int charScore = 10;

                if (inConsecutiveMatch && foundPos == match.matchPositions[match.matchPositions.size() - 2] + 1)
                {
                    consecutiveBonus += 5;
                    charScore += consecutiveBonus;
                }
                else
                {
                    consecutiveBonus = 0;
                    inConsecutiveMatch = true;
                }

                if (textIdx == 0 || !std::isalnum(lowerText[textIdx - 1]))
                {
                    charScore += 15;
                }

                if (text[textIdx] == query[queryIdx])
                {
                    charScore += 5;
                }

                match.score += charScore;
                queryIdx++;
                textIdx++; // Move past current match
            }

            // Apply penalties and validation
            if (queryIdx < lowerQuery.length())
            {
                match.score -= (lowerQuery.length() - queryIdx) * 3;
            }

            if (queryIdx < lowerQuery.length())
            {
                match.score = 0;
                match.matchPositions.clear();
            }

            return match;
        }

        // Original scalar implementation as fallback
        FuzzyMatch calculateMatchScalar(const std::string &text, const std::string &query)
        {
            FuzzyMatch match;
            match.text = text;
            match.score = 0;

            if (query.empty() || text.empty())
            {
                return match;
            }

            std::string lowerText = toLower(text);
            std::string lowerQuery = toLower(query);

            // Simple fuzzy matching algorithm
            size_t textIdx = 0;
            size_t queryIdx = 0;
            int consecutiveBonus = 0;
            bool inConsecutiveMatch = false;

            while (textIdx < lowerText.length() && queryIdx < lowerQuery.length())
            {
                if (lowerText[textIdx] == lowerQuery[queryIdx])
                {
                    match.matchPositions.push_back(textIdx);

                    // Base score for character match
                    int charScore = 10;

                    // Bonus for consecutive characters
                    if (inConsecutiveMatch)
                    {
                        consecutiveBonus += 5;
                        charScore += consecutiveBonus;
                    }
                    else
                    {
                        consecutiveBonus = 0;
                        inConsecutiveMatch = true;
                    }

                    // Bonus for word boundaries (start of word)
                    if (textIdx == 0 || !std::isalnum(lowerText[textIdx - 1]))
                    {
                        charScore += 15;
                    }

                    // Bonus for exact case match
                    if (text[textIdx] == query[queryIdx])
                    {
                        charScore += 5;
                    }

                    match.score += charScore;
                    queryIdx++;
                }
                else
                {
                    inConsecutiveMatch = false;
                    consecutiveBonus = 0;
                }
                textIdx++;
            }

            // Penalty for unmatched characters in query
            if (queryIdx < lowerQuery.length())
            {
                match.score -= (lowerQuery.length() - queryIdx) * 3;
            }

            // If we didn't match all query characters, score is 0
            if (queryIdx < lowerQuery.length())
            {
                match.score = 0;
                match.matchPositions.clear();
            }

            return match;
        }

        std::string toLower(const std::string &str)
        {
            std::string result = str;
            std::transform(result.begin(), result.end(), result.begin(), ::tolower);
            return result;
        }
    };
}