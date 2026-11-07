---
type: ImplementationDecision
title: Ler documento pelo número SEI via pesquisa rápida, árvore e link assinado do conteúdo
operation: documento.obter
implementation: partial
status: adopted-for-scope
scope: Refs #60 fatias 1 e 2; texto e metadados de documentos internos pelo número SEI, pelo caminho mais barato (--processo NUP pela abertura de processo NUP; índice número SEI → processo e documento pelo link da árvore, pelo link de trabalho ou pela entrada por id com documento da lista branca; pesquisa rápida), a árvore conferida (pastas abertas só se o documento não está à vista) e o Nos[i].src assinado documento_visualizar; lote em sequência; --saida com a foto DOC.vN.html 0600, idempotente (#207); recusa tipada para sem acesso, cancelado, externo, duplicidade e número diferente; --versao N, versao_atual e cópia de trabalho no apêndice de versões (Refs #88), sobre qualquer caminho; --com-propriedades e --ocultar-inacessiveis continuam stub
selected_strategy: web-indice-arvore-src-com-pesquisa-rapida
alternatives: web-indice-arvore-src-com-pesquisa-rapida,web-pesquisa-rapida-arvore-src,web-imprimir-web-da-barra,web-processo-indice-arvore-src,web-url-montada-por-id,web-pesquisa-avancada,rest-wssei
evidence_level: acervo-publico-sete-instalacoes-testes-sinteticos-e-live-unb
live_validation: pass-unb-4-0-9-fatias-1-e-2
---

# Ler documento pelo número SEI

## Alternativas consideradas

- **web-indice-arvore-src-com-pesquisa-rapida** (escolhida, fatia 2):
  `--processo NUP` pela abertura de `processo NUP`; senão o índice
  (número SEI → P e M) pelo link da árvore, pelo link de trabalho ou
  pela entrada por id com documento; senão, ou com o palpite
  desmentido, a pesquisa rápida da fatia 1.
- **web-pesquisa-rapida-arvore-src** (fatia 1): POST do número SEI na
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
- **`--processo NUP` e índice (fatia 2):** com o link da árvore guardado
  a leitura custa 2 (árvore e conteúdo); com o link de trabalho, 3; pela
  entrada com documento, 4 — o mesmo da pesquisa com `action` guardado
  (e 1 a menos que pelo Controle). A entrada vem antes da pesquisa: GET
  sem POST (a pesquisa pode ficar lembrada), forma da lista branca
  medida na UnB, aceite e recusa registrados por instalação, e só onde
  nenhuma das formas foi recusada. O 302 não leva direto à árvore: a
  tela é necessária para o link assinado dela, por isso 4 e não 2.
- **Pasta fechada:** aberta só se o número não está à vista. Medido na
  UnB: a tela com `id_documento` (entrada com documento ou pesquisa do
  número) já serve a árvore com a pasta do documento carregada; o link
  da árvore guardado e a abertura pelo NUP, não (+1).
- **Guardar o `.src`** e pular a árvore (1 requisição) rejeitado por ora:
  rótulo, natureza, pasta e processo vêm da árvore, e o `.src` sozinho
  não prova que o documento ainda está no processo.

Custo: **2** pelo link da árvore guardado, **3** pelo link de trabalho,
**4** pela entrada com documento ou pela pesquisa com `action` guardado,
**5** pelo Controle; **+1** com pasta fechada que esconde o documento;
palpite desmentido soma o que gastou e cai uma vez na pesquisa. Com
`--processo`, o custo de `processo NUP` + 1. Lote: soma por documento.

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
  `src/processo_obter/tests_documento.rs` e, da fatia 2, em
  `src/processo_obter/tests_documento_indice.rs` (sequências exatas de
  cada caminho) e `src/indice_local/tests_documentos.rs`; leitores em
  `src/arvore_processo/tests_conteudo.rs`; entidades em
  `src/entidades_html.rs` e `src/documento_web.rs`.
- RED observado com shims compiláveis: 12 testes falhando (leitores,
  fluxo e entidades), 1041 passando; GREEN: todos passam. Fatia 2: 16
  falhando (tabela `documento`, `nup_por_id`, migração e os caminhos do
  índice e de `--processo`), 1041 passando; GREEN: 1057.
- Replay do acervo `src/processo_obter/acervo_documento.rs` (ignorado no
  CI) e [relatório](../validacao-replay/2026-11-04-documento-obter-acervo.md).
- [Ensaio live na UnB](../validacao-live/2026-11-04-documento-obter.md).

## Omissão explícita na leitura do lote (Refs #60)

`--ocultar-inacessiveis` distingue ausência de conteúdo publicado
da recusa do próprio documento ou da sessão: apenas
`ConteudoNaoPublicadoNaArvore` é omitido. A contagem real de
requisições acompanha o erro tipado, e o lote prossegue sem retry.
Não publicar a identidade do omitido; demais falhas continuam visíveis.
BDD na CLI e testes sintéticos no agregador; ensaio específico live:
NOT_RUN nesta fatia (sem sessão autenticada de treinamento).

## Limitações e revisão

- Externos (`documento_download_anexo`): [apêndice](documento-obter/documento-externo.md);
  `--com-propriedades` e `--ocultar-inacessiveis`: não lidos.
  `--versao N` e `versao_atual`:
  [apêndice de versões](documento-obter/versoes.md) (Refs #88); a foto
  `DOC.vN.html` de `--saida`, que substituiu `DOC.txt` e a cópia de
  trabalho: [apêndice da foto](documento-obter/saida-idempotente.md) (#207).
- A entrada com documento só foi medida na UnB; noutra instalação o
  primeiro uso aprende (uma recusa derruba a sessão de todos e fica
  registrada).
- O texto é uma linha com os espaços colapsados; parágrafos e tabelas
  não são preservados.
- Lote com `--processo` abre o processo por documento (2 cada, com o
  link da árvore já guardado pelo primeiro).
- Rever se uma instalação responder a pesquisa por número SEI de outra
  forma, ou se o GET do conteúdo tiver efeito medido.
