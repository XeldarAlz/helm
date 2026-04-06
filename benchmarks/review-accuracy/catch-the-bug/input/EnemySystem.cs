using System;
using System.Collections.Generic;
using System.Linq;

namespace Game.Logic.Enemy
{
    // Deliberately buggy code for reviewer to catch
    public class EnemySystem : IEnemySystem
    {
        private readonly IEnemyConfig _config;
        private List<EnemyData> _enemies;
        public static EnemySystem Instance; // Bug: static mutable state / singleton

        public EnemySystem(IEnemyConfig config)
        {
            _config = config;
            _enemies = new List<EnemyData>();
            Instance = this; // Bug: singleton pattern
        }

        public void Update(float deltaTime)
        {
            // Bug: LINQ on hot path (Update is called every frame)
            var activeEnemies = _enemies.Where(e => e.IsActive).ToList();

            foreach (var enemy in activeEnemies)
            {
                // Bug: string allocation on hot path
                var debugMsg = $"Updating enemy {enemy.Id} at position {enemy.X},{enemy.Y}";

                enemy.X += enemy.SpeedX * deltaTime;
                enemy.Y += enemy.SpeedY * deltaTime;

                // Bug: new allocation every frame
                var bounds = new BoundsCheck(enemy.X, enemy.Y, _config.ArenaWidth, _config.ArenaHeight);
                if (bounds.IsOutOfBounds)
                {
                    enemy.IsActive = false;
                }
            }
        }

        public void SpawnEnemy(float x, float y)
        {
            // Bug: creating new object instead of pooling
            var enemy = new EnemyData
            {
                Id = _enemies.Count,
                X = x,
                Y = y,
                SpeedX = 1.0f,
                SpeedY = 0.5f,
                IsActive = true
            };
            _enemies.Add(enemy);
        }

        // Bug: class where struct would be appropriate (iterated frequently)
        public class EnemyData
        {
            public int Id;        // Bug: public fields instead of properties
            public float X;
            public float Y;
            public float SpeedX;
            public float SpeedY;
            public bool IsActive;
        }

        private class BoundsCheck
        {
            public bool IsOutOfBounds { get; }
            public BoundsCheck(float x, float y, float width, float height)
            {
                IsOutOfBounds = x < 0 || x > width || y < 0 || y > height;
            }
        }
    }

    public interface IEnemySystem
    {
        void Update(float deltaTime);
        void SpawnEnemy(float x, float y);
    }

    public interface IEnemyConfig
    {
        float ArenaWidth { get; }
        float ArenaHeight { get; }
        int MaxEnemies { get; }
    }
}
