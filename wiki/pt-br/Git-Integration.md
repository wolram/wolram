# Integração Git

WOLRAM integra com Git via o crate `git2` (bindings libgit2) para commitar automaticamente resultados de jobs e gerenciar branches.

## GitManager

A struct `GitManager` encapsula um `git2::Repository`:

```rust
let git = GitManager::open(".")?;
```

## Métodos

### `open(path) -> Result<GitManager>`

Abre um repositório Git existente no caminho fornecido.

### `commit(message) -> Result<String>`

Faz stage e commit de todos os arquivos rastreados e novos, retornando o hash curto de 7 caracteres.

**Arquivos excluídos** (não adicionados ao stage):
- `wolram.toml`
- `.env`
- `.env.local`
- `*.key`

**Autor do commit**: Usa o usuário Git configurado, ou cai para `WOLRAM <wolram@localhost>`.

### `commit_job_result(job) -> Result<String>`

Método de conveniência que formata uma mensagem de commit e chama `commit()`:

```
wolram: [habilidade] descrição (status)
```

Exemplo:
```
wolram: [code_generation] implementar função fibonacci (Completed)
```

### `create_branch(name) -> Result<()>`

Cria uma nova branch a partir de HEAD e faz checkout.

### `create_job_branch(job) -> Result<()>`

Cria uma branch nomeada com base no job:

```
wolram/<primeiros-8-caracteres-do-id-do-job>
```

Exemplo: `wolram/a1b2c3d4`

### `current_branch() -> Result<String>`

Retorna o nome da branch atual (formato abreviado).

## Integração com o Orquestrador

No fluxo do orquestrador, após o estado PROCESS completar com sucesso:

1. Se `has_git` é `true`, chama `GitManager::commit_job_result(job)`
2. Em caso de falha no commit, um aviso é impresso mas o job continua
3. O commit não afeta o status de sucesso/falha do job

Isso significa que a integração Git funciona no modelo best-effort — um erro de Git não causará a falha do job.

## Proteção de Arquivos Sensíveis

O método de commit automaticamente exclui arquivos sensíveis do staging:

| Padrão | Motivo |
|--------|--------|
| `wolram.toml` | Pode conter chaves de API |
| `.env` | Segredos de ambiente |
| `.env.local` | Segredos de ambiente local |
| `*.key` | Chaves privadas |

---

> *[English Version](../en/Git-Integration.md)*
