# 🌟 ZODIAKOS - Documentação Completa do Jogo

## 📋 Índice
1. [Visão Geral](#visão-geral)
2. [Recursos do Jogo](#recursos-do-jogo)
3. [Sistema de Estrelas](#sistema-de-estrelas)
4. [Sistema de Conexões](#sistema-de-conexões)
5. [Sistema de Especializações](#sistema-de-especializações)
6. [Sistema de Constelações](#sistema-de-constelações)
7. [Interface e Controles](#interface-e-controles)
8. [Mecânicas de Jogo](#mecânicas-de-jogo)
9. [Estratégias](#estratégias)

---

## 🎮 Visão Geral

**Zodiakos** é um jogo de colonização espacial e gerenciamento de recursos em 2D, desenvolvido em Rust usando o motor Bevy. O jogador gerencia uma rede de estrelas conectadas, coletando recursos, especializando sistemas estelares e formando constelações para maximizar a produção.

### Características Principais
- **Motor**: Bevy 0.14 (Rust)
- **Renderização**: 2D com efeitos de bloom/HDR
- **Gênero**: Estratégia em tempo real / Gerenciamento de recursos
- **Plataforma**: Windows/Linux/Mac

---

## 💎 Recursos do Jogo

### Tipos de Recursos

O jogo possui 9 tipos diferentes de recursos, cada um com sua cor e ícone característico:

| Recurso | Ícone | Cor | Categoria | Descrição |
|---------|-------|-----|-----------|-----------|
| **Water** | 💧 | Ciano | Vida Básica | Essencial para vida e agricultura |
| **Oxygen** | 🌬️ | Azul Claro | Vida Básica | Necessário para colonização |
| **Food** | 🌱 | Verde | Vida Básica | Sustento das colônias |
| **Iron** | 🪨 | Cinza | Minerais | Material de construção básico |
| **Copper** | ⚡ | Laranja | Minerais | Componentes elétricos |
| **Silicon** | 💻 | Branco-Azulado | Minerais | Tecnologia e computadores |
| **Uranium** | ☢️ | Verde Radioativo | Energia | Combustível nuclear |
| **Helium-3** | 🔋 | Dourado | Energia | Energia de fusão |
| **Energy Crystal** | ✨ | Roxo | Energia | Recurso raro de alta energia |

### Distribuição de Recursos
- **Estrela Inicial (Sol System)**: Recursos balanceados de todos os tipos básicos
- **Outras Estrelas**: 1-3 tipos de recursos aleatórios
- **Recursos Raros**: Energy Crystal e Helium-3 aparecem em quantidades menores

---

## ⭐ Sistema de Estrelas

### Propriedades das Estrelas

Cada estrela possui:
- **ID único** e **Nome procedural** (ex: "Alpha Centauri")
- **Recursos** com capacidade máxima
- **Taxa de produção base** (recursos/segundo)
- **Nível de especialização** (1 até infinito)
- **Cor baseada no recurso dominante**
- **Estado de colonização**
- **Lista de conexões** (entrada e saída)

### Estrela Inicial (Storage Hub)
- Nome: "Sol System (Storage Hub)"
- Função especial: **Hub de Armazenamento Central**
- Capacidade: 10x maior que estrelas normais
- Todas as rotas de suprimento devem levar a um Storage Hub
- Cor dourada especial

### Geração de Estrelas
- **12 estrelas** no total (incluindo Sol System)
- Posicionamento com **distância mínima de 90 unidades**
- Nomes gerados combinando prefixos (Alpha, Beta, Gamma...) com sufixos (Centauri, Orionis...)
- Cores HDR baseadas no recurso dominante para efeito bloom

---

## 🔗 Sistema de Conexões

### Criação de Conexões
- **Click e arraste** de uma estrela para outra
- Linha visual durante o arraste
- Conexão permanente ao soltar

### Limite de Conexões (Fibonacci)
O número máximo de conexões de saída segue a **sequência de Fibonacci** baseada no nível da estrela:

| Nível | Max Conexões | Sequência |
|-------|--------------|-----------|
| 1 | 1 | F(1) |
| 2 | 1 | F(2) |
| 3 | 2 | F(3) |
| 4 | 3 | F(4) |
| 5 | 5 | F(5) |
| 6 | 8 | F(6) |
| 7 | 13 | F(7) |
| 8 | 21 | F(8) |
| 9 | 34 | F(9) |
| 10+ | ... | Continua... |

### Coleta de Recursos
- Timer de **5 segundos** por ciclo de coleta
- Recursos são transferidos da estrela conectada para o jogador
- Taxa de coleta afetada pela distância ao Storage Hub

### Distância e Eficiência
A eficiência da produção depende da distância (em saltos) até o Storage Hub mais próximo:

| Distância | Eficiência | Status |
|-----------|------------|---------|
| 0 saltos | 100% | É um Storage Hub |
| 1 salto | 90% | Adjacente (Ótimo) |
| 2 saltos | 75% | Rota Curta (Bom) |
| 3 saltos | 60% | Rota Média |
| 4 saltos | 45% | Rota Longa (Subótimo) |
| 5 saltos | 35% | Rota Muito Longa |
| 6+ saltos | <30% | Rota Crítica (Mínimo 10%) |
| Sem rota | 10% | Isolado (Crítico) |

---

## 🏭 Sistema de Especializações

### Tipos de Especialização

| Especialização | Ícone | Tempo Build | Função |
|----------------|-------|-------------|---------|
| **None** | ⛏️ | 5s | Extração de recursos (padrão) |
| **Storage** | 📦 | 10s | Hub de armazenamento (10x capacidade) |
| **Military** | 🚀 | 20s | Produz naves de guerra |
| **Mining** | ⚒️ | 15s | Produz naves mineradoras |
| **Agriculture** | 🌾 | 12s | Produz comida e recursos biológicos |
| **Research** | 🔬 | 25s | Produz cientistas e tecnologia |
| **Medical** | 🏥 | 15s | Produz unidades médicas |
| **Industrial** | 🏭 | 18s | Produz construtores |

### Mecânicas de Especialização
- Estrelas especializadas **não extraem recursos** (exceto Storage)
- Consomem recursos para produzir **unidades especializadas**
- Custos de produção diminuem com níveis mais altos (20% mais eficiente por nível)
- Tempo de upgrade: `tempo_base * nível * 1.5`

### Unidades Produzidas

| Tipo | Produzido por | Quantidade/Ciclo |
|------|---------------|------------------|
| Warship | Military | 1x nível |
| MiningShip | Mining | 2x nível |
| Farmer | Agriculture | 3x nível |
| Scientist | Research | 1x nível |
| Doctor | Medical | 2x nível |
| Builder | Industrial | 2x nível |
| StorageModule | Storage | 1x nível |

### Estados de Construção
- **Ready**: Operacional
- **Building**: Construindo especialização (mostra progresso %)
- **Upgrading**: Fazendo upgrade de nível (mostra progresso %)

---

## 🌌 Sistema de Constelações

### Formação de Constelações
- Formadas quando **3 ou mais estrelas** criam um **ciclo fechado**
- Detecção automática usando algoritmo DFS (Depth-First Search)
- Visual: Mesh 2D poligonal com cor única + linhas brilhantes

### Restrições
- ⚠️ **Uma estrela pode pertencer a apenas UMA constelação**
- Primeira constelação formada tem prioridade
- Não é possível sobrepor constelações
- Sistema impede formação de novas constelações com estrelas já utilizadas

### Benefícios
- 🎯 **Bônus de 100% na produção** (2x) para todas as estrelas da constelação
- Indicação visual na UI: "⭐ CONSTELLATION BONUS: 2x Production! ⭐"
- Efeito visual permanente com cor única para cada constelação

### Aspectos Visuais
- **Mesh 2D customizado** seguindo exatamente o formato do ciclo
- Preenchimento poligonal semi-transparente
- Linhas de contorno com 3 camadas de glow:
  - Externa: muito larga e transparente (α=0.15)
  - Média: largura média (α=0.3)
  - Central: fina e brilhante (α=0.9)
- Cores geradas usando ângulo dourado para boa distribuição

---

## 🎛️ Interface e Controles

### Controles do Mouse
- **Click esquerdo**: Selecionar estrela ou conexão
- **Click + Arraste**: Criar conexão entre estrelas
- **Hover**: Destacar estrela (borda branca)

### Controles do Teclado

#### Especialização (com estrela selecionada)
- **1**: Resource Extraction
- **2**: Storage Hub
- **3**: Military Base
- **4**: Mining Station
- **5**: Agricultural Colony
- **6**: Research Center
- **7**: Medical Facility
- **8**: Industrial Complex
- **U**: Upgrade nível da especialização
- **DELETE**: Remover conexão selecionada

#### Menu de Configuração Bloom
- **ESC**: Abrir/fechar menu
- **Q/A**: Ajustar intensidade do bloom
- **W/S**: Ajustar threshold
- **E/D**: Ajustar low frequency boost
- **R**: Resetar valores padrão

### Interface (HUD)

#### Painel de Recursos (Esquerda)
Mostra todos os recursos do jogador:
```
=== RESOURCES ===
💧 Water: 150.5
🌬️ Oxygen: 89.3
🌱 Food: 234.1
...
```

#### Painel de Informações (Direita)
Informações detalhadas da estrela/conexão selecionada:
- Nome e ID da estrela
- Status de colonização
- Especialização e nível
- Estado de construção/upgrade
- Conexões atuais/máximas
- Distância ao Storage Hub
- Eficiência de produção
- Recursos disponíveis
- Bônus de constelação

---

## ⚙️ Mecânicas de Jogo

### Fluxo de Jogo Principal

1. **Início**: Jogador começa com Sol System (Storage Hub) colonizado
2. **Exploração**: Visualizar estrelas e seus recursos
3. **Expansão**: Criar conexões para coletar recursos
4. **Especialização**: Converter estrelas para produção especializada
5. **Otimização**: Formar constelações para bônus de produção
6. **Evolução**: Fazer upgrade de níveis para mais conexões

### Sistema de Produção

#### Estrelas de Extração (None)
```
Produção Efetiva = Taxa_Base × Modificador_Distância × Bônus_Constelação
```

#### Estrelas Especializadas
```
Custo_Produção = Custo_Base × (1 / (1 + (nível-1) × 0.2))
Unidades_Produzidas = Tipo_Base × Nível
```

### Rotas de Suprimento
- Todas as estrelas precisam de rota até um Storage Hub
- Algoritmo BFS para calcular menor distância
- Eficiência decresce com distância
- Estrelas isoladas produzem apenas 10%

---

## 🎯 Estratégias

### Estratégias Básicas

1. **Hub Central**
   - Manter Storage Hub central na rede
   - Minimizar distância média das estrelas

2. **Especialização Gradual**
   - Começar com extração de recursos
   - Especializar após acumular recursos suficientes

3. **Constelações Precoces**
   - Formar triângulos pequenos primeiro
   - Expandir para ciclos maiores depois

### Estratégias Avançadas

1. **Maximização de Constelações**
   - Planejar ciclos maiores (6-8 estrelas)
   - Evitar desperdício com ciclos pequenos
   - Considerar recursos antes de "trancar" estrelas

2. **Rede Fibonacci Ótima**
   - Fazer upgrade estratégico de níveis
   - Hubs de nível alto no centro da rede
   - Folhas de nível baixo nas extremidades

3. **Cadeia de Produção**
   - Storage Hubs distribuídos estrategicamente
   - Especializações complementares próximas
   - Rotas curtas para produção crítica

4. **Eficiência de Distância**
   - Máximo 3 saltos do Storage Hub
   - Criar Storage Hubs secundários se necessário
   - Reconfigurar conexões conforme a rede cresce

### Dicas Importantes

- ⚠️ **Constelações são permanentes** - planejar antes de formar
- 📦 **Storage Hubs múltiplos** podem melhorar eficiência global
- 🔄 **Reconectar estrelas** pode otimizar rotas de suprimento
- 📈 **Níveis altos** demoram mais mas aumentam drasticamente as conexões
- 🎯 **Bônus de constelação (2x)** é mais valioso que reduzir distância

---

## 🔧 Aspectos Técnicos

### Tecnologias Utilizadas
- **Linguagem**: Rust
- **Motor**: Bevy 0.14
- **Renderização**: WebGL2/Vulkan com HDR e Bloom
- **Mesh 2D**: Customizado para constelações
- **UI**: Sistema de texto nativo do Bevy

### Arquitetura ECS
- **Entities**: Estrelas, Conexões, UI elements
- **Components**: Star, Connection, ConstellationMarker, etc.
- **Systems**: Atualização, coleta, detecção de ciclos, UI
- **Resources**: PlayerResources, GameState, ConstellationTracker

### Performance
- Detecção de ciclos otimizada com cache
- Limite de estrelas para manter 60 FPS
- Bloom configurável para diferentes hardwares

---

## 📝 Créditos

Desenvolvido com Rust e Bevy por [Seu Nome]
Conceito original de colonização espacial com mecânicas únicas de constelações e progressão Fibonacci.

---

*Versão 1.0 - Setembro 2025*