using System.Linq;
using Cysharp.Threading.Tasks;
using UnityEngine;
using Random = UnityEngine.Random;

namespace Play
{
    public class UnityChan : MonoBehaviour
    {
        [Tooltip("How long does it take for Unity-Chan to get boring in seconds")]
        [SerializeField] private float boringInterval = 10f;
        
        [Tooltip("The audios to play in random when Unity-Chan is greeting")]
        [SerializeField] private AudioClip[] greetingAudios;
        
        [Tooltip("The audios to play in random when Unity-Chan is thinking")]
        [SerializeField] private AudioClip[] thinkingAudios;
        
        [Tooltip("The audios to play in the order of boring actions when Unity-Chan is boring")]
        [SerializeField] private AudioClip[] boringAudios;
        
        [Tooltip("The audios to play in random when Unity-Chan is spawning a disk")]
        [SerializeField] private AudioClip[] spawningAudio;
        
        [Tooltip("The audios to play in random when Unity-Chan won the game")]
        [SerializeField] private AudioClip[] wonAudios;
        
        [Tooltip("The audios to play in random when Unity-Chan lost the game")]
        [SerializeField] private AudioClip[] lostAudios;
        
        [Tooltip("The audios to play in random when draw the game")]
        [SerializeField] private AudioClip[] drawAudios;

        private AudioSource _audioSource;
        private Animator _animator;
        private int _thinkingHash;
        private int _wonHash;
        private int _drawHash;
        private int _lostHash;
        private int _isBoringHash;
        private int _boringIndexHash;
        private int _idleStateHash;
        private int[] _boringStateHashes;

        private Vector3 _startPosition;

        private Board _board;
        private bool IsIdle
        {
            get
            {
                return _animator.GetCurrentAnimatorStateInfo(0).shortNameHash == _idleStateHash;
            }
        }
        
        private void Awake()
        {
            _audioSource = GetComponent<AudioSource>();
            _animator = GetComponentInChildren<Animator>();
            _thinkingHash = Animator.StringToHash("thinking");
            _wonHash = Animator.StringToHash("won");
            _drawHash = Animator.StringToHash("draw");
            _lostHash = Animator.StringToHash("lost");
            _isBoringHash = Animator.StringToHash("isBoring");
            _boringIndexHash = Animator.StringToHash("boringIndex");
            _idleStateHash = Animator.StringToHash("Idle");
            _boringStateHashes = new[]
            {
                Animator.StringToHash("Boring"),
                Animator.StringToHash("Boring 1"),
            };

            _startPosition = transform.localPosition;

            _board = FindObjectOfType<Board>(true);
            
            _board.OnThinking += OnThinking;
            _board.OnDecided += OnDecided;
            _board.OnGameOver += OnGameOver;
        }

        private void Start()
        {
            var choice = Random.Range(0, greetingAudios.Length);
            _audioSource.PlayOneShot(greetingAudios[choice]);

            StartCoroutine(SetBoring().ToCoroutine());
        }

        private async UniTask SetBoring()
        {
            while (true)
            {
                await UniTask.WaitUntil(() => IsIdle);
                
                await UniTask.WaitForSeconds(boringInterval);
                if (!IsIdle)
                {
                    _animator.SetBool(_isBoringHash, false);
                    continue;
                }
            
                var choice = Random.Range(0, boringAudios.Length);

                _animator.SetInteger(_boringIndexHash, choice);
                _animator.SetBool(_isBoringHash, true);
                
                await UniTask.WaitUntil(() => _boringStateHashes.Contains(_animator.GetCurrentAnimatorStateInfo(0).shortNameHash));

                _animator.SetBool(_isBoringHash, false);
                _audioSource.Stop();
                _audioSource.PlayOneShot(boringAudios[choice]);
            }
        }

        private async UniTask OnThinking()
        {
            _animator.SetBool(_isBoringHash, false);
            _animator.SetBool(_thinkingHash, true);
            
            await UniTask.WaitUntil(() => _animator.IsInTransition(0));

            _audioSource.Stop();
            var choice = Random.Range(0, thinkingAudios.Length);
            _audioSource.PlayOneShot(thinkingAudios[choice]);
        }

        private async UniTask OnDecided(Tile tile)
        {
            _animator.SetBool(_isBoringHash, false);
            _animator.SetBool(_thinkingHash, false);
            
            // Wait for playing spawn animation
            await UniTask.WaitUntil(() => _animator.IsInTransition(0));
            await UniTask.WaitForSeconds(0.25f);
            
            _audioSource.Stop();
            var choice = Random.Range(0, spawningAudio.Length);
            _audioSource.PlayOneShot(spawningAudio[choice]);
            
            transform.localPosition = _startPosition;
        }

        private UniTask OnGameOver(Player? winner)
        {
            _audioSource.Stop();
            switch (winner)
            {
                case Player.Bot:
                {
                    var choice = Random.Range(0, wonAudios.Length);
                    _audioSource.PlayOneShot(wonAudios[choice]);
                    _animator.SetTrigger(_wonHash);
                    break;
                }
                case Player.Human:
                {
                    var choice = Random.Range(0, lostAudios.Length);
                    _audioSource.PlayOneShot(lostAudios[choice]);
                    _animator.SetTrigger(_lostHash);
                    break;
                }
                default:
                {
                    var choice = Random.Range(0, drawAudios.Length);
                    _audioSource.PlayOneShot(drawAudios[choice]);
                    _animator.SetTrigger(_drawHash);
                    break;
                }
            }
            return UniTask.CompletedTask;
        }
    }
}