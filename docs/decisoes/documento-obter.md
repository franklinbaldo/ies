---
type: ImplementationDecision
title: Ler documento pelo número SEI via pesquisa rápida, árvore e link assinado do conteúdo
operation: documento.obter
implementation: partial
status: adopted-for-scope
scope: Refs #60 fatia 1; texto e metadados de documentos internos pelo número SEI, pela pesquisa rápida (action guardado ou do Controle), a tela que seleciona o documento, a árvore conferida e o Nos[i].src assinado documento_visualizar; lote em sequência; --saida com DOC.txt 0600; recusa tipada para sem acesso, cancelado, externo, duplicidade e número diferente; --versao N, versao_atual e cópia de trabalho no apêndice de versões (Refs #88); --processo, --com-propriedades e --ocultar-inacessiveis continuam stub
selected_strategy: web-pesquisa-rapida-arvore-src
alternatives: web-pesquisa-rapida-arvore-src,web-imprimir-web-da-barra,web-processo-indice-arvore-src,web-url-montada-por-id,web-pesquisa-avancada,rest-wssei
evidence_level: acervo-publico-sete-instalacoes-testes-sinteticos-e-live-unb
live_validation: pass-unb-4-0-9
---

# Ler documento pelo número SEI

## Alternativas consideradas

- **web-pesquisa-rapida-arvore-src** (escolhida): POST do número SEI na
  pesquisa rápida → 302 com o `id_protocolo` do documento → tela cujo
  `iframe#ifrArvore` seleciona esse `id_documento` → árvore → GET do
  `Nos[i].src` (`documento_visualizar`) assinado do nó.
- **web-imprimir-web-da-barra:** o link `documento_imprimir_web` que a
  barra (`Nos[i].acoes`) do documento publica, e que o Todos usou.
- **web-processo-indice-arvore-src:** com `--processo NUP`, abrir a
  árvore pelo caminho mais barato de `processo NUP` (1 GET com o link
  guardado) e seguir o `.src`; o índice aprenderia DOC ↔ `id_documento`.
- **web-url-montada-por-id:** montar `documento_visualizar` pelo id.
- **web-pesquisa-avancada:** `protocolo_pesquisar` com o número.
- **rest-wssei:** fora da prioridade ([ADR 0001](../arquitetura/0001-rotas-por-evidencia.md)).

## Comparação

- **Pesquisa rápida por número SEI provada nas sete instalações.** As
  oito cassettes `documento-numero-e-busca-*` e
  `documento-propriedades-unb` (UnB, UFRJ, GO, PEN1 a PEN4) gravaram o
  POST com 302 para `procedimento_trabalhar&id_protocolo=<id do
  documento>` e a tela com `id_procedimento=<processo>&id_documento=<id>`
  na árvore. É o mesmo formulário e o mesmo 302 que `processo NUP` já lê.
- **`.src` contra a barra.** O `.src` é publicado por nó, para todo
  documento, sem precisar selecioná-lo; a barra é a do nó e muda por
  permissão. Na `bloco-ciclo-unb` (passos 17 e 18) as duas respostas
  do mesmo documento têm o mesmo título; a impressão traz
  `window.print()` no `onload` do `body`. Escolhido o `.src`.
- **URL do conteúdo montada rejeitada:** a [ADR 0003](../arquitetura/0003-indice-local.md)
  registra que `documento_visualizar`, `documento_imprimir_web` e
  `documento_versao_listar` sem assinatura derrubam a sessão de todos. A
  lista branca da mesma ADR aceita
  `procedimento_trabalhar&id_procedimento=P&id_documento=M` (302 para a
  forma assinada, documento selecionado): adotada na fatia 2, quando o
  índice souber `P` e `M` do número SEI (2 requisições em vez de 4).
- **Pesquisa avançada:** custa uma requisição a mais e a lista não é lida
  (decisão de [`processo.obter`](processo-obter.md)).
- **`--processo NUP`:** mais barato (2 GET com o link da árvore
  guardado) e precisa de uma tabela de documentos no índice; fica para a
  fatia 2. Sem ele, o número SEI sozinho não leva ao processo.

Custo: **4** requisições com o `action` da pesquisa guardado, **5** sem
ele (Controle). Lote: soma por documento.

## Decisão

`ies documento DOC [DOC ...]` sem `--html-privado` segue o
[contrato da leitura Web](../../contracts/cli/documento-obter-web.md).
Leitores novos sem regex: `Nos[i].src` pelo leitor de literais da
árvore, a tela do documento e a conferência comum dos endereços com a
chave `id_documento`. O número SEI é opaco: `11505` não é `0011505`. O
texto reusa o leitor do `--html-privado`, agora com as entidades HTML
decodificadas (medido ao vivo: `&ccedil;`, `&nbsp;`, `&ordm;`).

**Forma do `.src` no acervo** ([replay](../validacao-replay/2026-11-04-documento-obter-acervo.md)):
interno e e-mail → `documento_visualizar`; PDF, TXT e HTML recebido →
`documento_download_anexo`; cancelado, ZIP e interno sem acesso → `''`.
O transporte só aceita `text/html` até 8 MB: externos ficam para outra
fatia, com recusa tipada.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/documento-obter.md) e
  [da leitura Web](../../contracts/cli/documento-obter-web.md)
- [Matriz por instalação](../../contracts/evidencias/documento-obter.md)
- [Testes de superfície](../../tests/cli_contract.rs)
- Testes do fluxo com transporte roteirizado em
  `src/processo_obter/tests_documento.rs`; leitores em
  `src/arvore_processo/tests_conteudo.rs`; entidades em
  `src/entidades_html.rs` e `src/documento_web.rs`.
- RED observado com shims compiláveis: 12 testes falhando (leitores,
  fluxo e entidades), 1041 passando; GREEN: todos passam.
- Replay do acervo `src/processo_obter/acervo_documento.rs` (ignorado no
  CI) e [relatório](../validacao-replay/2026-11-04-documento-obter-acervo.md).
- [Ensaio live na UnB](../validacao-live/2026-11-04-documento-obter.md).

## Limitações e revisão

- Externos (`documento_download_anexo`), `--processo`,
  `--com-propriedades`, `--ocultar-inacessiveis` e documento em pasta
  fechada: não lidos. `--versao N`, `versao_atual` e a cópia de trabalho:
  [apêndice de versões](documento-obter/versoes.md) (Refs #88).
- O texto é uma linha com os espaços colapsados; parágrafos e tabelas
  não são preservados.
- Repetir um documento custa 4 requisições: o índice ainda não guarda
  DOC ↔ `id_documento` nem o `.src` (fatia 2).
- Rever se uma instalação responder a pesquisa por número SEI de outra
  forma, ou se o GET do conteúdo tiver efeito medido.
