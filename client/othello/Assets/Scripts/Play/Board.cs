using System.Collections;
using UnityEngine;

namespace Play
{
    public class Board : MonoBehaviour
    {
        [Tooltip("The game object for communicating with the AI server")]
        [SerializeField] private Bot bot;
        
        private Grid _grid;

        private void Awake()
        {
            _grid = GetComponentInChildren<Grid>();
        }
        
        private IEnumerator Start()
        {
            yield return InitializeBoard();
            _grid.OnDiskPlaced += OnDiskPlaced;
        }

        private IEnumerator InitializeBoard()
        {
            yield return bot.InitialBoard(response =>
            {
                _grid.Enumerate((i, j, tile) =>
                {
                    if (!DiskColorMethods.CanParse(response[i][j]))
                    {
                        return;
                    }
                    tile.PlaceDisk(DiskColorMethods.Parse(response[i][j]));
                });
            });
        }
        
        private IEnumerator OnDiskPlaced(Tile tile)
        {
            yield return bot.Result(_grid, Player.Human, tile, UpdateGrid);
            yield return new WaitForSeconds(1);
            yield return bot.Decide(_grid, (decision, result, winner) =>
            {
                decision.PlaceDisk(Player.Bot.Disk());
                UpdateGrid(result);

                if (!winner.HasValue)
                {
                    return;
                }
                
                Debug.Log("Game over");
                Debug.Log(PlayerMethods.CanParse(winner.Value) ? $"Winner: {PlayerMethods.Parse(winner.Value)}" : "Winner: Draw");
            });
        }
        
        private void UpdateGrid(char[][] newGrid)
        {
            _grid.Enumerate((i, j, current) =>
            {
                if (!DiskColorMethods.CanParse(newGrid[i][j]))
                {
                    if (current.Disk != null)
                    {
                        current.ClearDisk();
                    }
                    return;
                }
                    
                var diskColor = DiskColorMethods.Parse(newGrid[i][j]);

                if (current.Disk == null)
                {
                    current.PlaceDisk(diskColor);
                } 
                else if (current.Disk.Color != diskColor)
                {
                    current.Disk.Flip();
                }
            });
        }
    }
}