using System;
using System.Collections.Generic;
using Cysharp.Threading.Tasks;
using UnityEngine;
using UnityEngine.Assertions;

namespace Play
{
    public class Board : MonoBehaviour
    {
        [Tooltip("The wrapper for the AI server")]
        [SerializeField] private Bot bot;
        
        private BoardGrid _grid;
        
        /// <summary>
        /// The duration to wait in seconds until the next flip animation starts for further disk
        /// </summary>
        private const float FlipBreakDuration = 0.2f;

        public delegate UniTask GridUpdate();
        public event GridUpdate OnGridUpdate;
        
        public delegate UniTask Thinking();
        public event Thinking OnThinking;
        
        public delegate UniTask Decided(Tile tile);
        public event Decided OnDecided;
        
        public delegate UniTask GameOver(Player? winner);
        public event GameOver OnGameOver;

        private void Awake()
        {
            _grid = GetComponentInChildren<BoardGrid>();
            _grid.OnDiskPlaced += OnDiskPlaced;
        }
        
        private async void Start()
        {
            var response = await bot.InitialBoard();
            await UpdateGrid(response, new BoardGrid.Position(0, 0));
            await UpdateActiveTiles();
        }
        
        private async UniTask OnDiskPlaced(BoardGrid.Position position)
        {
            var tile = _grid.Tile(position);
            if (!tile.CanPlaceDisk)
            {
                return;
            }

            foreach (var t in _grid.Tiles())
            {
                t.CanPlaceDisk = false;
            }

            var result = await bot.Result(_grid, Player.Human, position);
            await UpdateGrid(result.Board, position);

            if (result.IsGameOver)
            {
                await OnGameOver?.Invoke(result.Winner).ToCoroutine();
                return;
            }
            
            await Decide();
        }

        private async UniTask Decide()
        {
            await OnThinking?.Invoke().ToCoroutine();
            var decision = await bot.Decide(_grid);
            var tile = _grid.Tile(decision.Position);
            await OnDecided?.Invoke(tile).ToCoroutine();
            
            if (decision.Position == null)
            {
                Debug.Log("AI has no actions to take this turn");
                await UpdateActiveTiles();
                return;
            }
            
            tile.PlaceDisk(Player.Bot.Disk());
            
            await _grid.WaitWhileUpdating();
            await UpdateGrid(decision.Result.Board, decision.Position);
            
            if (decision.Result.IsGameOver)
            {
                await OnGameOver?.Invoke(decision.Result.Winner).ToCoroutine();
                return;
            }
            
            await UpdateActiveTiles();
        }
        
        private async UniTask UpdateGrid(char[][] newGrid, BoardGrid.Position start)
        {
            var flipping = new List<Tuple<uint, Tile>>();
            foreach (var (i, j, current) in _grid.Enumerate(start))
            {
                if (!DiskColorMethods.CanParse(newGrid[i][j]))
                {
                    if (current.Disk != null)
                    {
                        current.ClearDisk();
                    }
                    continue;
                }
                    
                var diskColor = DiskColorMethods.Parse(newGrid[i][j]);

                if (current.Disk == null)
                {
                    current.PlaceDisk(diskColor);
                } 
                else if (current.Disk.Color != diskColor)
                {
                    flipping.Add( new Tuple<uint, Tile>(
                        start.Distance(new BoardGrid.Position(i, j)),
                        current
                        )
                    );
                }
            }
            
            await _grid.WaitWhileUpdating();

            uint prevDistance = 1;
            foreach (var (distance, tile) in flipping)
            {
                if (distance > prevDistance)
                {
                    await UniTask.WaitForSeconds(FlipBreakDuration);
                }
                
                Debug.Log($"Flipping {tile.name}");
                Assert.IsNotNull(tile.Disk);
                tile.Disk.Flip();

                prevDistance = distance;
            }
            
            await _grid.WaitWhileUpdating();
            Debug.Log("Finished updating board grid");
            await OnGridUpdate?.Invoke().ToCoroutine();
        }

        private async UniTask UpdateActiveTiles()
        {
            var actions = await bot.Actions(_grid);
            if (actions.Count == 0)
            {
                Debug.Log("Human has no actions to take this turn");
                await Decide();
            }
            
            foreach (var tile in _grid.Tiles())
            {
                tile.CanPlaceDisk = actions.Contains(BoardGrid.Position.Parse(tile));
            }
        }
    }
}