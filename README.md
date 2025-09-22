# Zodiakos - Game 2D com Rust e Bevy

Este é um projeto de game 2D desenvolvido com Rust usando o engine Bevy.

## Funcionalidades atuais

- **Bloom 2D**: Implementação do efeito de pós-processamento bloom que adiciona um brilho realista aos objetos luminosos

## Como executar

```bash
cargo run
```

## Controles do Bloom

- **Space**: Liga/desliga o efeito bloom
- **A**: Alterna entre modos Energy-conserving e Additive
- **Q/E**: Ajusta a intensidade do bloom
- **W/S**: Ajusta o Low Frequency Boost
- **R/F**: Ajusta a curvatura do Low Frequency Boost
- **T/G**: Ajusta a frequência do High Pass
- **Y/H**: Ajusta o threshold
- **U/J**: Ajusta o soft threshold (knee)

## Estrutura do Projeto

```
zodiakos/
├── Cargo.toml          # Configurações e dependências do projeto
├── src/
│   └── main.rs        # Código principal do game
└── assets/
    └── bevy_bird_dark.png  # Sprite do pássaro Bevy
```

## Próximos passos

- [ ] Adicionar sistema de movimento para o jogador
- [ ] Implementar sprites e animações customizadas
- [ ] Adicionar sistema de física
- [ ] Criar mecânicas de jogo específicas
- [ ] Implementar sistema de áudio
- [ ] Adicionar UI e menus

## Dependências

- **Bevy 0.14**: Engine de jogos moderno escrito em Rust
- Dynamic linking habilitado para compilação mais rápida em desenvolvimento# zodiakos
