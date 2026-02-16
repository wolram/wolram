# Contribuindo

## Configuração do Ambiente de Desenvolvimento

1. Clone o repositório
2. Instale Rust (edição 2024) via [rustup](https://rustup.rs/)
3. Execute `cargo build` para verificar a configuração
4. Execute `cargo test` para confirmar que todos os testes passam

## Convenções

### Mensagens de Commit

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: adicionar nova habilidade para migrações de banco de dados
fix: tratar rate limit sem header retry-after
docs: atualizar diagrama de arquitetura
test: adicionar testes mock para caminho LLM do roteador
refactor: simplificar lógica de transição da máquina de estados
```

### Estilo de Código

- Execute `cargo fmt` antes de commitar
- Execute `cargo clippy` e corrija todos os avisos
- Todos os tipos públicos devem derivar `Serialize` e `Deserialize`
- Toda execução de job deve produzir um `AuditRecord`

### Testes

- Escreva testes como módulos `#[cfg(test)]` inline nos arquivos fonte
- Use `MockClient` para testes que precisam de respostas LLM
- Use `tempfile::TempDir` para testes que precisam de um repositório Git
- Marque testes assíncronos com `#[tokio::test]`
- Busque cobertura abrangente tanto de caminhos felizes quanto de casos de erro

### Nomenclatura de Branches

- Branches de funcionalidade: `feat/<descrição>`
- Correções de bugs: `fix/<descrição>`
- Branch padrão: `main`

## Diretrizes de Arquitetura

- **Pureza da máquina de estados**: `StateMachine::next()` deve permanecer uma função pura que recebe um job e resultado, retornando uma transição. Efeitos colaterais pertencem ao orquestrador.
- **Abstração baseada em traits**: Use traits (como `MessageSender`) para permitir testes sem dependências externas.
- **Propagação de erros**: Use `anyhow::Result` para erros de aplicação e `thiserror` para tipos de erro em nível de biblioteca.
- **Serialização**: Todos os tipos de dados que cruzam fronteiras devem derivar `Serialize`/`Deserialize`.

## Adicionando uma Nova Habilidade

1. Adicione o nome da habilidade e palavras-chave ao `SkillRouter` em `router.rs`
2. Atualize o prompt de `classify_with_llm()` para incluir a nova habilidade
3. Adicione testes para o novo roteamento por palavras-chave
4. Atualize a página [Roteador e Habilidades](Router-and-Skills.md) desta wiki

## Adicionando um Novo Estado

1. Adicione a variante ao enum `State` em `state_machine/state.rs`
2. Atualize a lógica de transição em `StateMachine::next()`
3. Adicione o código de tratamento em `orchestrator.rs`
4. Adicione testes cobrindo todas as transições envolvendo o novo estado
5. Atualize a página [Máquina de Estados](State-Machine.md) desta wiki

---

> *[English Version](../en/Contributing.md)*
