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

## Retomada de 05/11 — revisão do leitor Web e validação do head

Esta seção **atualiza** as limitações descritas acima: desde a primeira
fatia offline, o branch passou a incluir a coleta Web integral e o fallback
na ausência de snapshot; isso não equivale a validação ao vivo.

- **PR/branch:** [#250](https://github.com/franklinbaldo/ies/pull/250),
  `codex/244-unidades-indice-local`, ainda DRAFT.
- **Correções de segurança:** URLs de ações servidas pela lupa passam por
  validação da instalação/origem, unicidade de `acao` e `infra_hash` válido,
  e rejeição de chave não canônica, separador ambíguo ou hash duplicado.
  A URL do Controle conserva a validação própria do transporte. O botão
  `btnNovo` duplicado não pode escolher um destino arbitrário.
- **Erros tipados:** `FalhaDaLupa::Transporte` preserva a
  `FalhaTransporte` completa até a CLI, incluindo `status_http` e
  `mensagem_do_servidor` saneada quando a falha HTTP dispõe desses dados.
  Nenhuma URL assinada nem cookie entra na saída.
- **Testes sintéticos adicionais:** URLs válidas e duplicadas, consulta
  malformada, proteção contra segundo GET após link recusado, HTTP 503
  com dados de apresentação, botão repetido. O teste de CLI para
  `--atualizar` sem sessão foi atualizado para exigir recusa de configuração
  **antes** de qualquer acesso Web, em vez de esperar a recusa antiga
  de funcionalidade pendente.
- **Orçamento sintético:** 4 GETs de navegação e 1 POST por página;
  em duas páginas, 6 requisições, sem retry. A comprovação em instalações
  de treinamento continua pendente.
- **CI canônico:** **PASS** no commit `32d178929f5e7bd80706f8c3806c7d555c8a0fc0`,
  [run 42070714695](https://github.com/franklinbaldo/ies/actions/runs/42070714695).
  Rustfmt, Clippy, suíte Rust, limite 20 KB, OKF, decisões e subcódigos
  passaram. Runs intermediários vermelhos por seletor HTML `#btnNovo`,
  Clippy, Rustfmt e expectativa legada foram diagnosticados e corrigidos.
- **RED local:** NOT_RUN; contagem local não observada. O sandbox não
  dispõe de Rust e a rede/DNS impede instalar o toolchain. A evidência
  histórica de **3 falhas no CI antes do GREEN** continua sendo
  [run 42063259438](https://github.com/franklinbaldo/ies/actions/runs/42063259438),
  sem descrevê-la como RED local desta retomada.
- **Replay do acervo:** NOT_RUN; pendente em cassettes reais.
- **Live UnB:** NOT_RUN — 0 requisições.
- **Live GO:** NOT_RUN — 0 requisições.
- **Escrita no SEI:** NOT_RUN — nenhum objeto criado ou alterado, resíduo
  nenhum; esta issue implementa leitura.

A execução do CI citada comprova o commit indicado, **não** automaticamente
commits subsequentes no mesmo branch. Para fechar a issue faltam o replay
sobre cassettes reais e a leitura ao vivo em UnB/GO com orçamento contado,
confirmação das telas e comportamento de todas as páginas. Não fazer merge
nem marcar a PR como pronta enquanto isso não estiver observado.

## Retomada adicional — impedir snapshot filtrado que parece integral

**Observação anterior ao RED:** o oráculo Todos @`323eb01`, em
`src/todos/sei/lupa_unidades.py` e
`tests/test_lupa_unidades_010.py`, mostra a distinção entre o órgão
`Todos` (valor vazio) e `null` (nenhum órgão), além dos filtros de
sigla e descrição. `tests/test_destinos_indice_010.py` exige que uma
lista apenas filtrada não seja guardada como snapshot integral.
A observação vem do oráculo, **não** de replay da cassette no IES.

**Risco corrigido:** mesmo com número de páginas, total e IDs
coerentes, o SEI poderia devolver uma consulta filtrada por critérios
restaurados na sessão. O leitor agora verifica os filtros que a resposta
de **cada POST** efetivamente publica, antes de considerar uma página
para persistência: sigla e descrição vazias, órgão selecionado `Todos`
e controles sem duplicação. Qualquer divergência recusa a coleta inteira
sem substituir o snapshot anterior e sem fazer POST adicional. A
regressão sintética também cobre órgão `null` e seleção implícita.

| Verificação | Resultado |
| --- | --- |
| RED local pré-código | **NOT_RUN**, sem toolchain Rust no sandbox e DNS externo indisponível |
| RED no CI, antes da correção | **1 teste falhou**, 1.358 passaram, 12 ignorados — [run 42073006096](https://github.com/franklinbaldo/ies/actions/runs/42073006096), commit `f42f6ab`; a falha foi `filtro_restaurado_pelo_sei_nao_vira_lista_integral` |
| GREEN do código/testes | **PASS** (Rustfmt, Clippy, suíte Rust e demais gates) — [run 42074031892](https://github.com/franklinbaldo/ies/actions/runs/42074031892), commit `7d31cf9` |
| Replay da cassette original | **NOT_RUN**, corpos da cassette não disponíveis no sandbox |
| Live UnB e GO | **NOT_RUN**, **0 requisições** em ambas |
| Escrita remota | **NOT_RUN**, **0 requisições**, nenhum resíduo criado |

**Limite:** os novos testes reproduzem o cenário observado pelo Todos
em HTML **sintético**, mas não substituem a execução integral da cassette
de GO ou a observação autenticada em UnB e GO. Antes de concluir #244,
reproduzir as páginas do acervo e medir as requisições de
`unidade listar --atualizar` e a pesquisa posterior em treinamento,
sem retry e sem produção. Este relatório não afirma PASS live.

## Retomada 🦊 — navegação segura e replay GO

- **RED (somente CI):** 2 falhas antes da correção do botão `btnNovo`, [run 42110481074](https://github.com/franklinbaldo/ies/actions/runs/42110481074). O leitor emprestava link assinado de outro controle ou declarava ambiguidade indevida. Agora a URL vem apenas do próprio `btnNovo`.
- **RED (somente CI):** 1 falha no leitor da lista vazia antes da correção, [run 42111627207](https://github.com/franklinbaldo/ies/actions/runs/42111627207). O POST 12 da cassette GO apresenta `frmUnidadeLista` e `Nenhum registro encontrado.`, sem tabela.
- **Gates:** PASS no head `dc9fbf8`, [run 42112344927](https://github.com/franklinbaldo/ies/actions/runs/42112344927). **Replay:** PASS, seis respostas GO (3 GET, 3 POST), ZIP público conferido, [run 42112345185](https://github.com/franklinbaldo/ies/actions/runs/42112345185). Zero requisições reais ao SEI nesse replay.
- **Sandbox Rust / RED local:** NOT_RUN (sem toolchain e sem DNS para instalar). **Live UnB:** NOT_RUN, 0 requisições. **Live GO:** NOT_RUN, 0 requisições. **Pesquisa live:** NOT_RUN. Nenhuma escrita, nenhum resíduo remoto.
- Ainda faltam ensaios autenticados em UnB/GO, cobertura da coleta integral de múltiplas páginas, integração com a `main` e remoção dos workflows temporários antes de integrar a PR.


## Nova fatia 🦊 após a integração da PR #250 — observação pré-RED

**Fonte observada (código já publicado na main, não resposta live):**
`src/processo_enviar_cli.rs` associa `sugestoes_guardadas` ao erro
`DestinoNaoOferecidoPeloSei` quando encontra sugestões do snapshot
SQLite. Porém a apresentação humana é delegada a
`processo_concluir_cli::publicar_valor_da_falha`, que exibe
`erro`, `subcodigo`, `aberto_em`, `depois`, `falha_do_envio`
e `verificacao`, **não** `sugestoes_guardadas`. Assim as sugestões
ficam acessíveis no modo JSON, mas invisíveis no modo humano.
A correção terá escopo limitado a `processo enviar`; o SEI
continuará sendo a autoridade sobre o destino efetivamente oferecido.

**Acervo observado:** o card
`cassettes/go/unidade-listar-destinos-go--965da599b3e4.md`
identifica o ZIP público com SHA-1 fixo. O teste Rust existente
`unidade_lupa::acervo::replay_da_lupa_go_no_acervo` confere o membro
e processa seis respostas (3 GET e 3 POST) sem contato com o SEI,
mas ainda está fora do CI canônico. A nova PR integra o replay
à execução obrigatória do mesmo `ci.yml`. Este replay não demonstra
coleta integral de 274 páginas e não substitui live UnB/GO.
