---
type: SEIObservation
title: Implementação offline de listagem e pesquisa de unidades no índice local
description: Issue #244; teste RED no CI, implementação de snapshot SQLite e ensaio sintético de ponta a ponta; nenhum contato com instalações reais
---

# Handoff da #244 — lista de unidades, fatia offline (05/11/2026)

## Escopo implementado

- `ies unidade listar [--json]`: abre apenas o índice SQLite local,
  valida a identidade da instalação e a unidade da sessão local e
  publica o snapshot já guardado (`origem: indice`, `lida_em`,
  `total`, `unidades`, `requisicoes: 0`).
- `ies unidade pesquisar TERMO [--json]`: filtra esse snapshot por sigla
  ou descrição, ignorando acento e caixa; sigla exata primeiro.
  Nenhuma requisição Web.
- A ausência da lista, índice indisponível, unidade não identificada,
  `--unidade` não verificável e `--atualizar` ainda não disponível
  falham com causas tipadas, **sem inventar resultado Web**.
- Testes da CLI incluem um banco SQLite e arquivos privados de sessão
  **sintéticos**, snapshot vazio de outra unidade, acentos e saída JSON.
- Contratos OKF, decisões e matriz incluem os dois novos comandos parciais.
  A tabela SQLite continua pública apenas para `SELECT`, como previsto.

## RED e GREEN

| Etapa | Resultado | Evidência |
| --- | --- | --- |
| Observação pré-RED | PASS documental | [Lupa observada no Todos](../como-o-sei-funciona/lupa-de-unidades.md), não é captura autenticada desta sessão |
| RED local | **NOT_RUN** | Sandbox sem `cargo`/`rustc`; DNS externo indisponível |
| RED no CI | **3 testes falharam, 0 passaram** | [Run 42063259438](https://github.com/franklinbaldo/ies/actions/runs/42063259438), commit `0b07e6d`, antes da integração da CLI |
| GREEN de escopo inicial | **PASS** no CI | [Run 42064554487](https://github.com/franklinbaldo/ies/actions/runs/42064554487), commit `b5e8f29`, gate canônico completo |
| Testes SQLite E2E | Resultados no CI da PR | [PR #250](https://github.com/franklinbaldo/ies/pull/250); acompanhar o último head, que também precisa do gate completo |
| Gate local `prek` | **NOT_RUN** | Ferramentas Rust ausentes, sem acesso DNS para instalação |

O GREEN do commit anterior não substitui a verificação do head final.
Os testes adicionados antes do código falharam no CI, não no sandbox:
não descrever como RED local.

## Validação real e orçamento

| Etapa | Veredito | Requisições |
| --- | --- | ---: |
| Consultas sintéticas ao SQLite, sem Web | PASS no CI quando `cli_unidade_indice_cache` passar | 0 |
| `unidade listar --atualizar` na UnB | **NOT_RUN** | 0 |
| `unidade listar --atualizar` em Goiás | **NOT_RUN** | 0 |
| `unidade pesquisar` ao vivo | **NOT_RUN** | 0 |
| Replay da lupa gravada no acervo IES | **NOT_RUN** | 0 |

Não houve login nem GET/POST na Web do SEI nesta execução.
O orçamento para `unidade listar --atualizar` permanece pendente:
GETs até a lupa + GET da lupa + POST por página, sem repetição, com
verificação de integridade antes de substituir o snapshot.

## Faltas e próximo passo

1. Ler as cassettes reais da lupa, implementar derivação do formulário
   com `formulario_web`, conferência de páginas/IDs/totais e coleta
   completa sem gravar listas parciais.
2. Conectar `--atualizar` e o preenchimento automático quando a lista
   está ausente; quando o índice estiver desligado, permitir somente
   leitura Web e avisar que não foi persistida.
3. Aplicar sugestões de sigla do snapshot no `processo enviar --para`
   sem consulta extra, sem mudar a escolha final pelo autocomplete.
4. Replay das cassettes e ensaios de leitura UnB/GO, com contagem de
   requisições. Nunca repetir requisição falha; respeitar intervalos e
   proibições do `AGENTS.md`.

**PR Draft:** [#250](https://github.com/franklinbaldo/ies/pull/250).
Nenhuma escrita remota foi executada, nem foi feita afirmação de PASS live.
