# Arquitetura

## Visão Geral

WOLRAM segue uma arquitetura modular organizada em torno de uma máquina de estados central. O sistema foi projetado para orquestrar tarefas de desenvolvimento assistidas por IA com garantias de confiabilidade inspiradas no Robotic Enterprise Framework (REFramework) da UiPath.

## Diagrama do Sistema

```
┌─────────────────────────────────────────────────────┐
│                      CLI (cli.rs)                   │
│         run | demo | status | flags globais         │
└─────────────────────┬───────────────────────────────┘
                      │
              ┌───────▼────────┐
              │  Orquestrador  │
              │ (orchestrator) │
              └──┬──────┬──┬──┘
                 │      │  │
     ┌───────────┘      │  └───────────┐
     │                  │              │
┌────▼─────┐   ┌───────▼──────┐  ┌────▼────┐
│ Roteador │   │  Máquina de  │  │ Cliente │
│ (router) │   │   Estados    │  │Anthropic│
└──────────┘   │(state_machine)│ └─────────┘
     │         └──────────────┘        │
     │              ┌──────────────────┘
     │              │
┌────▼─────┐  ┌────▼────┐
│   Git    │  │   UI    │
│  (git)   │  │  (ui)   │
└──────────┘  └─────────┘
```

## Mapa de Módulos

| Módulo | Arquivo(s) | Responsabilidade |
|--------|------------|------------------|
| `state_machine` | `src/state_machine/mod.rs`, `job.rs`, `state.rs` | Tipos centrais: `Job`, `State`, `Transition`, `StateMachine`, `AuditRecord`, `ModelTier`, `RetryConfig`, `FailureKind` |
| `cli` | `src/cli.rs` | CLI baseado em clap: subcomandos `run`, `demo`, `status` com flags globais |
| `config` | `src/config.rs` | `WolramConfig` carregado de `wolram.toml` com sobrescritas de variáveis de ambiente |
| `orchestrator` | `src/orchestrator.rs` | `JobOrchestrator::run_job()` — conduz um job por todos os 4 estados |
| `router` | `src/router.rs` | `SkillRouter` (atribuição por palavras-chave/LLM) e `ModelSelector` (seleção por complexidade) |
| `anthropic` | `src/anthropic/mod.rs`, `client.rs`, `types.rs`, `error.rs` | Cliente HTTP para a API de Mensagens da Anthropic |
| `git` | `src/git.rs` | `GitManager`: commit e criação de branches via libgit2 |
| `ui` | `src/ui.rs` | `JobProgress`: spinner + saída colorida via indicatif/console |

## Fluxo de Dados

1. **CLI** analisa os argumentos e carrega a configuração
2. **Orquestrador** cria um `Job` e inicia o loop da máquina de estados
3. **Roteador** atribui uma habilidade e nível de modelo (via palavras-chave ou classificação LLM)
4. **Máquina de Estados** computa transições baseadas nos resultados
5. **Cliente Anthropic** envia prompts e recebe respostas (ou stubs no modo offline)
6. **Gerenciador Git** commita resultados ao completar com sucesso
7. **UI** fornece feedback em tempo real durante todo o processo
8. **Registro de Auditoria** é produzido ao final de cada job

## Dependências

| Crate | Papel |
|-------|-------|
| `tokio` | Runtime assíncrono |
| `clap` | Parsing de argumentos CLI |
| `serde` + `serde_json` | Serialização |
| `toml` | Parsing de arquivo de configuração |
| `reqwest` | Cliente HTTP para API da Anthropic |
| `git2` | Bindings libgit2 |
| `anyhow` | Propagação de erros |
| `thiserror` | Derive de Error para `AnthropicError` |
| `chrono` | Timestamps |
| `uuid` | Geração de ID de jobs |
| `indicatif` | Spinner de progresso no terminal |
| `console` | Saída colorida no terminal |

### Dependências de Desenvolvimento

| Crate | Papel |
|-------|-------|
| `tempfile` | Diretórios temporários para testes de Git |
| `wiremock` | Servidor HTTP mock para testes de cliente |

---

> *[English Version](../en/Architecture.md)*
