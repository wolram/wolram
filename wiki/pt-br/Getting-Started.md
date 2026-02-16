# Primeiros Passos

## Pré-requisitos

- **Rust** (edição 2024) — Instale via [rustup](https://rustup.rs/)
- **libgit2** — Necessário pelo crate `git2` (geralmente incluído automaticamente)
- **Chave de API Anthropic** (opcional) — Necessária para chamadas reais ao LLM; sem ela, os jobs rodam em modo stub

## Instalação

Clone o repositório e compile:

```bash
git clone https://github.com/wolram/wolram.git
cd wolram
cargo build
```

## Configuração

### Chave de API

Defina sua chave de API da Anthropic como variável de ambiente:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

Alternativamente, crie um arquivo `wolram.toml` na raiz do projeto (veja [Configuração](Configuration.md) para detalhes).

### Arquivo de Configuração (Opcional)

Copie o exemplo de configuração:

```bash
cp wolram.toml.example wolram.toml
```

Edite `wolram.toml` para definir padrões de nível de modelo, máximo de retries e delay de backoff.

## Executando Seu Primeiro Job

### Modo Demo

Execute a demo integrada da máquina de estados para ver o ciclo de vida completo:

```bash
cargo run -- demo
```

Isso percorre um job de exemplo por todos os quatro estados (INIT → DEFINE_AGENT → PROCESS → END) e imprime o registro de auditoria.

### Executando um Job Real

Execute um job por descrição:

```bash
cargo run -- run "implementar uma função que calcula números de fibonacci"
```

Se `ANTHROPIC_API_KEY` estiver configurada, isso chamará a API da Anthropic. Caso contrário, roda em modo stub com sucesso simulado.

### Carregando um Job de Arquivo

Você também pode carregar um job pré-configurado de um arquivo JSON:

```bash
cargo run -- run --file job.json
```

### Verificando o Status

Veja a configuração atual, status da chave de API e informações do repositório Git:

```bash
cargo run -- status
```

## Flags do CLI

Todos os subcomandos aceitam estas flags globais:

| Flag | Descrição |
|------|-----------|
| `--model haiku\|sonnet\|opus` | Sobrescreve a seleção automática de nível de modelo |
| `--max-retries <n>` | Sobrescreve o número máximo de retries |
| `--verbose` / `-v` | Ativa a saída detalhada no stderr |

Exemplo com sobrescritas:

```bash
cargo run -- --model opus --max-retries 5 --verbose run "redesenhar o módulo de autenticação"
```

## Próximos Passos

- Leia o guia de [Arquitetura](Architecture.md) para entender o design do sistema
- Explore a [Máquina de Estados](State-Machine.md) para aprender sobre ciclos de vida dos jobs
- Consulte a [Referência CLI](CLI-Reference.md) para todos os comandos disponíveis

---

> *[English Version](../en/Getting-Started.md)*
