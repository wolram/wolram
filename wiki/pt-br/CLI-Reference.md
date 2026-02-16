# Referência CLI

WOLRAM fornece uma interface de linha de comando baseada em clap com três subcomandos e flags globais.

## Uso

```
wolram [OPÇÕES] <COMANDO>
```

## Flags Globais

Estas flags estão disponíveis em todos os subcomandos:

| Flag | Curta | Descrição |
|------|-------|-----------|
| `--model <NÍVEL>` | | Sobrescreve a seleção de nível de modelo. Valores: `haiku`, `sonnet`, `opus` |
| `--max-retries <N>` | | Sobrescreve o número máximo de retries (padrão: 3) |
| `--verbose` | `-v` | Ativa saída detalhada no stderr |

## Subcomandos

### `run`

Executa um job. Forneça uma descrição inline ou carregue de um arquivo JSON.

```bash
# Executar por descrição
wolram run "implementar uma função de fibonacci"

# Executar a partir de um arquivo JSON
wolram run --file job.json

# Com sobrescritas
wolram --model opus --max-retries 5 run "redesenhar o módulo de autenticação"
```

**Argumentos:**

| Argumento | Obrigatório | Descrição |
|-----------|-------------|-----------|
| `<description>` | Não* | Texto de descrição do job |
| `--file <caminho>` | Não* | Caminho para um arquivo Job serializado em JSON |

\* Um entre `description` ou `--file` deve ser fornecido.

**Comportamento ao carregar arquivo**: Ao carregar de `--file`, os seguintes campos são resetados para garantir um estado inicial limpo:
- `state` → `Init`
- `retry_count` → `0`
- `state_history` → `[]`
- `status` → `Pending`

### `demo`

Executa a demo integrada da máquina de estados. Percorre um job de exemplo por todos os quatro estados e imprime o registro de auditoria.

```bash
wolram demo
```

Sem argumentos ou opções.

### `status`

Exibe a configuração atual, status da chave de API e informações do repositório Git.

```bash
wolram status
```

Mostra:
- Configuração carregada de `wolram.toml`
- Se `ANTHROPIC_API_KEY` está definida
- Branch atual e informações do repositório Git

## Exemplos

```bash
# Execução básica de job
cargo run -- run "adicionar testes unitários para o módulo de roteamento"

# Modo detalhado com sobrescrita de modelo
cargo run -- --verbose --model haiku run "corrigir erro de digitação no README"

# Walkthrough da demo
cargo run -- demo

# Verificar status do sistema
cargo run -- status

# Carregar job de arquivo com máximo de retries
cargo run -- --max-retries 10 run --file job-complexo.json
```

## Códigos de Saída

| Código | Significado |
|--------|-------------|
| 0 | Sucesso |
| 1 | Job falhou (após retries esgotados ou erro de validação) |

---

> *[English Version](../en/CLI-Reference.md)*
