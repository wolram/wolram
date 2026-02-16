# Configuração

WOLRAM é configurado através de uma combinação de arquivo de configuração TOML e variáveis de ambiente.

## Arquivo de Configuração: `wolram.toml`

Coloque um arquivo `wolram.toml` na raiz do projeto. Se ausente, todos os valores padrão são usados.

### Exemplo

```toml
api_key = ""
default_model_tier = "sonnet"
max_retries = 3
base_delay_ms = 1000
```

Um template está disponível em `wolram.toml.example`.

### Campos

| Campo | Tipo | Padrão | Descrição |
|-------|------|--------|-----------|
| `api_key` | String | `""` | Chave de API da Anthropic. Sobrescrita pela variável de ambiente `ANTHROPIC_API_KEY` |
| `default_model_tier` | String | `"sonnet"` | Nível de modelo padrão (`haiku`, `sonnet`, `opus`) |
| `max_retries` | u32 | `3` | Máximo de tentativas de retry para o estado PROCESS |
| `base_delay_ms` | u64 | `1000` | Delay base em milissegundos para backoff exponencial |

## Variáveis de Ambiente

| Variável | Descrição | Prioridade |
|----------|-----------|------------|
| `ANTHROPIC_API_KEY` | Chave de API da Anthropic | Sobrescreve `api_key` no `wolram.toml` |

### Definindo a Chave de API

```bash
# Linux/macOS
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Ou inline
ANTHROPIC_API_KEY="sk-ant-..." cargo run -- run "sua tarefa"
```

## Sobrescritas via CLI

Flags do CLI têm a maior prioridade, sobrescrevendo tanto o arquivo de configuração quanto as variáveis de ambiente:

```bash
# Sobrescrever nível de modelo
cargo run -- --model opus run "tarefa complexa"

# Sobrescrever máximo de retries
cargo run -- --max-retries 10 run "tarefa instável"
```

## Ordem de Prioridade

Para cada configuração, a fonte de maior prioridade vence:

```
Flags do CLI  >  Variáveis de ambiente  >  wolram.toml  >  Padrões
```

## Modo Stub

Quando nenhuma chave de API está configurada (string vazia e sem variável `ANTHROPIC_API_KEY`), WOLRAM roda em **modo stub**:

- O orquestrador pula chamadas reais à API
- Jobs completam com sucesso simulado
- Todas as outras funcionalidades (máquina de estados, roteamento, git, auditoria) funcionam normalmente
- Útil para testes e desenvolvimento sem custos de API

---

> *[English Version](../en/Configuration.md)*
