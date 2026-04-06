namespace Game.Logic.Score
{
    public interface IScoreProcessor
    {
        void Process(ReadOnlySpan<ScoreEntry> entries, Span<ScoreResult> results);
        int MaxCapacity { get; }
    }

    public readonly struct ScoreEntry
    {
        public readonly int PlayerId;
        public readonly int RawScore;
        public readonly float Multiplier;

        public ScoreEntry(int playerId, int rawScore, float multiplier)
        {
            PlayerId = playerId;
            RawScore = rawScore;
            Multiplier = multiplier;
        }
    }

    public struct ScoreResult
    {
        public int PlayerId;
        public int FinalScore;
        public int Rank;
    }

    public interface IScoreConfig
    {
        int MaxEntries { get; }
        int BatchSize { get; }
    }
}
