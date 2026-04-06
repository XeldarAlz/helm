namespace Game.Logic.Score
{
    public interface IScoreSystem
    {
        void AddScore(int playerId, int points);
        ReadOnlySpan<ScoreEntry> GetTopScores(int count);
        int GetPlayerScore(int playerId);
        void ResetScores();
        int TotalEntries { get; }
    }

    public interface IScoreConfig
    {
        int MaxEntries { get; }
        int MaxScorePerEntry { get; }
    }

    public interface IScoreEvents
    {
        void OnScoreAdded(int playerId, int points, int newTotal);
        void OnScoresReset();
    }

    public readonly struct ScoreEntry
    {
        public readonly int PlayerId;
        public readonly int Score;

        public ScoreEntry(int playerId, int score)
        {
            PlayerId = playerId;
            Score = score;
        }
    }
}
