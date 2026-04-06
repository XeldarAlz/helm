using System;
using System.Collections.Generic;

namespace Game.Logic.Score
{
    public sealed class ScoreSystem : IScoreSystem
    {
        private readonly IScoreConfig _config;
        private readonly IScoreEvents _events;
        private readonly Dictionary<int, int> _scores;
        private readonly ScoreEntry[] _sortBuffer;
        private int _entryCount;

        public int TotalEntries => _entryCount;

        public ScoreSystem(IScoreConfig config, IScoreEvents events)
        {
            _config = config ?? throw new ArgumentNullException(nameof(config));
            _events = events ?? throw new ArgumentNullException(nameof(events));
            _scores = new Dictionary<int, int>(config.MaxEntries);
            _sortBuffer = new ScoreEntry[config.MaxEntries];
        }

        public void AddScore(int playerId, int points)
        {
            if (points < 0)
            {
                throw new ArgumentOutOfRangeException(nameof(points), "Score cannot be negative");
            }

            if (_scores.TryGetValue(playerId, out int existing))
            {
                int newTotal = Math.Min(existing + points, _config.MaxScorePerEntry);
                _scores[playerId] = newTotal;
                _events.OnScoreAdded(playerId, points, newTotal);
            }
            else
            {
                if (_entryCount >= _config.MaxEntries)
                {
                    return;
                }

                int clamped = Math.Min(points, _config.MaxScorePerEntry);
                _scores[playerId] = clamped;
                _entryCount++;
                _events.OnScoreAdded(playerId, points, clamped);
            }
        }

        public ReadOnlySpan<ScoreEntry> GetTopScores(int count)
        {
            int resultCount = Math.Min(count, _entryCount);
            int bufferIndex = 0;

            foreach (KeyValuePair<int, int> kvp in _scores)
            {
                _sortBuffer[bufferIndex++] = new ScoreEntry(kvp.Key, kvp.Value);
            }

            Array.Sort(_sortBuffer, 0, bufferIndex, ScoreComparer.Instance);
            return new ReadOnlySpan<ScoreEntry>(_sortBuffer, 0, resultCount);
        }

        public int GetPlayerScore(int playerId)
        {
            return _scores.TryGetValue(playerId, out int score) ? score : 0;
        }

        public void ResetScores()
        {
            _scores.Clear();
            _entryCount = 0;
            _events.OnScoresReset();
        }

        private sealed class ScoreComparer : IComparer<ScoreEntry>
        {
            public static readonly ScoreComparer Instance = new();
            public int Compare(ScoreEntry x, ScoreEntry y) => y.Score.CompareTo(x.Score);
        }
    }
}
