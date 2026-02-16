# Wiki WOLRAM

**WOLRAM** é uma camada de orquestração de nível empresarial para desenvolvimento assistido por IA, escrita em Rust (edição 2024). Aplica padrões de máquina de estados no estilo REFramework — inspirado no Robotic Enterprise Framework da UiPath — a workflows de codificação com LLM, com roteamento inteligente de modelo/habilidade, lógica de retry e trilha de auditoria completa.

## Status Atual

**v0.1.0** — Máquina de estados, CLI, cliente HTTP para Anthropic, roteador de habilidades, integração com Git e interface de terminal estão implementados. O cliente Anthropic funciona quando `ANTHROPIC_API_KEY` está configurada; caso contrário, os jobs rodam em modo stub (sucesso simulado).

## Funcionalidades Principais

- **Orquestração por Máquina de Estados** — Todo job segue um ciclo de vida bem definido: INIT → DEFINE_AGENT → PROCESS → END
- **Roteamento Inteligente** — Atribuição automática de habilidade e seleção de nível de modelo baseada na complexidade da tarefa
- **Retry com Backoff Exponencial** — Lógica de retry configurável para falhas de negócio e de sistema
- **Trilha de Auditoria Completa** — Toda execução de job produz um `AuditRecord` estruturado com estimativa de custo, temporização e transições de estado
- **Integração com Git** — Commit automático dos resultados de jobs via libgit2
- **Interface de Terminal** — Spinners de progresso e saída colorida para feedback em tempo real

## Páginas da Wiki

| Página | Descrição |
|--------|-----------|
| [Primeiros Passos](Getting-Started.md) | Instalação, configuração e primeira execução |
| [Arquitetura](Architecture.md) | Design do sistema e visão geral dos módulos |
| [Máquina de Estados](State-Machine.md) | Tipos, transições e fluxo da máquina de estados |
| [Referência CLI](CLI-Reference.md) | Comandos, subcomandos e flags |
| [Configuração](Configuration.md) | `wolram.toml` e variáveis de ambiente |
| [Cliente Anthropic](Anthropic-Client.md) | Cliente HTTP para a API de Mensagens da Anthropic |
| [Roteador e Habilidades](Router-and-Skills.md) | Atribuição de habilidades e seleção de nível de modelo |
| [Integração Git](Git-Integration.md) | Commits automáticos e gerenciamento de branches |
| [Testes](Testing.md) | Estratégia de testes, execução e escrita de novos testes |
| [Contribuindo](Contributing.md) | Convenções de desenvolvimento e guia de contribuição |

## Links Rápidos

- [Repositório GitHub](https://github.com/wolram/wolram)
- [Licença: MIT](https://github.com/wolram/wolram/blob/main/LICENSE)

---

> *[English Version](../en/Home.md)*
