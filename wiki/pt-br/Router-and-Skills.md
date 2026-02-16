# Roteador e Habilidades

O módulo de roteamento toma duas decisões para cada job: **qual habilidade** atribuir e **qual nível de modelo** usar.

## Roteador de Habilidades

`SkillRouter::route(description) -> String` atribui um rótulo de habilidade baseado na descrição do job.

### Habilidades Disponíveis

| Habilidade | Descrição |
|------------|-----------|
| `testing` | Escrita de testes, specs, suítes de teste |
| `refactoring` | Reestruturação e limpeza de código |
| `documentation` | Documentação, READMEs, comentários |
| `bug_fix` | Depuração, correção de erros |
| `code_generation` | Novas funcionalidades, implementações (padrão) |

### Pontuação por Palavras-chave

O roteador usa correspondência ponderada de palavras-chave:

| Habilidade | Palavras-chave (peso) |
|------------|----------------------|
| `testing` | "test" (10), "spec" (5) |
| `refactoring` | "refactor" (10), "clean up" (5) |
| `documentation` | "doc" (10), "readme" (5) |
| `bug_fix` | "fix" (10), "bug" (10), "debug" (7), "error" (5) |
| `code_generation` | "implement" (5), "add" (3), "create" (5), "build" (5) |

A descrição é convertida para minúsculas e escaneada para cada palavra-chave. A habilidade com a maior pontuação acumulada vence. Se nenhuma palavra-chave corresponder, `code_generation` é usado como padrão.

### Classificação via LLM

Quando uma chave de API está disponível, `classify_with_llm()` tenta a classificação via LLM primeiro:

1. Envia a descrição do job para `claude-haiku-4-5-20251001` com um prompt estruturado
2. Espera JSON: `{"skill": "...", "complexity": "..."}`
3. Em caso de sucesso, retorna a habilidade e avaliação de complexidade do LLM
4. Em caso de falha (rede, JSON inválido, habilidade desconhecida), cai para pontuação por palavras-chave

## Seletor de Modelo

`ModelSelector::select(description) -> ModelTier` escolhe um nível de modelo baseado na complexidade da tarefa.

### Pontuação de Complexidade

Duas categorias de palavras-chave são pontuadas:

**Palavras-chave simples** (direcionam para Haiku):

| Palavra-chave | Peso |
|---------------|------|
| "rename" | 10 |
| "format" | 10 |
| "typo" | 10 |
| "delete" | 7 |
| "remove" | 5 |
| "update" | 3 |

**Palavras-chave complexas** (direcionam para Opus):

| Palavra-chave | Peso |
|---------------|------|
| "architect" | 10 |
| "redesign" | 10 |
| "overhaul" | 10 |
| "refactor" | 8 |
| "migrate" | 8 |
| "multi-file" | 10 |
| "system" | 5 |

### Heurísticas

Além das palavras-chave, o seletor aplica:

- **Heurística de comprimento**: Descrições com menos de 20 caracteres ganham +5 na pontuação simples; mais de 100 caracteres ganham +5 na pontuação complexa
- **Contagem de palavras**: Mais de 15 palavras adiciona +3 na pontuação complexa

### Lógica de Seleção

| Condição | Resultado |
|----------|-----------|
| pontuação_simples > pontuação_complexa + 5 | `Haiku` |
| pontuação_complexa > pontuação_simples + 5 | `Opus` |
| Caso contrário | `Sonnet` (padrão) |

## Fluxo de Integração

Na fase DEFINE_AGENT do orquestrador:

1. Se um cliente de API está disponível:
   - Tenta `classify_with_llm()` primeiro
   - Em caso de falha, cai para roteamento por palavras-chave
2. Se não há cliente de API:
   - Usa `SkillRouter::route()` e `ModelSelector::select()` diretamente
3. Se uma sobrescrita via CLI `--model` foi fornecida, ela substitui o nível selecionado
4. A habilidade e o modelo são armazenados em `job.agent` como um `AgentConfig`

---

> *[English Version](../en/Router-and-Skills.md)*
