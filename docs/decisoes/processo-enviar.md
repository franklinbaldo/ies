---
type: ImplementationDecision
title: Envio do processo pela tela que a barra publica, com o destino do autocomplete e verificação pela situação da árvore
operation: processo.enviar
implementation: partial
status: adopted-for-scope
scope: ies processo enviar NUP --para SIGLA [--manter-aberto] com prévia que pede a tela Enviar Processo e a consulta do autocomplete declarada nela, até três com sigla com / (órgão do prefixo, órgão da tela, termo alargado; nenhum POST de escrita); --confirmar com um POST derivado pelo primitivo (hdnUnidades com o item escolhido) e verificação pelo destino e pela unidade da sessão em aberto_em de uma leitura nova da árvore
selected_strategy: web-tela-e-autocomplete-verificada-pela-situacao
alternatives: web-tela-e-autocomplete-verificada-pela-situacao,web-previa-sem-tela,web-lupa-de-unidades,web-verificada-pelo-historico,rest-wssei
evidence_level: cassettes-publicas-sete-instalacoes-todos-go-e-testes-sinteticos
live_validation: previa-ufrj-0415-pass-go-503-busca-sem-destino-escrita-not-run
---

# Enviar o processo (Refs #173)

## Contexto

Escrita central do fluxo do SEI, depois de
[`processo concluir`](processo-concluir.md). A barra publica um link
(forma única nas sete instalações); a tela de envio **não lista
destinos**: a unidade entra pelo autocomplete que o script declara
(`infraAjaxAutoCompletar` → `controlador_ajax.php`), observação em
[envio do processo](../como-o-sei-funciona/envio-do-processo.md). A
issue pede "prévia sem POST" e `--para` só com destino observado na tela;
as duas coisas juntas exigem a consulta, que é um POST de leitura.

## Alternativas consideradas

1. **Tela e autocomplete na prévia, verificação pela situação**
   (escolhida): GET da tela, POST da consulta (efeito avaliado:
   consulta), escolha pela sigla exata; `--confirmar` acrescenta um POST e
   a leitura nova; sucesso só se o destino e a origem ficaram como pedido.
2. **Prévia sem tela**: só a abertura, como `concluir`; a sigla não seria
   conferida contra o que o SEI oferece e o id do destino ficaria para o
   `--confirmar` (a prévia não diria o que vai ser feito).
3. **Lupa de unidades** (`unidade_selecionar_envio_processo`): janela à
   parte com formulário e paginação; nenhuma gravação de uso, e no GO
   (64 órgãos) uma lista enorme.
4. **Verificação pelo histórico** ("Processo remetido pela unidade", a do
   Todos): mais uma leitura (o histórico), e o reenvio às mesmas unidades
   também gera linha; `aberto_em` já é lido pela árvore.
5. **REST/WSSEI**: fora da prioridade ([ADR 0001](../arquitetura/0001-rotas-por-evidencia.md)).

## Comparação

| Critério | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- |
| Destino conferido na prévia | sim | não | sim | sim | — |
| Instalações cobertas pela evidência | 7 | 7 | 0 | 7 | 0 |
| Prévia a frio | 4 GET + 1 POST (consulta) | 3 GET | ? | 4 GET + 1 POST | — |
| `--confirmar` a frio | 5 GET + 2 POST | 5 GET + 2 POST | ? | 6 GET + 2 POST | — |
| URL montada pelo IES | nenhuma | nenhuma | nenhuma | nenhuma | — |

## Decisão

- Abertura, cookie, GET da tela, leitura de verificação e contagem são
  as peças de `processo_concluir::escrita`; a abertura lê o link de
  enviar com as outras ações da barra (`AcoesDeEstado::enviar`).
- A consulta é um primitivo novo e pequeno do formulário
  (`derivar_consulta_ajax`): só `controlador_ajax.php` da instalação
  (`Instalacao::autoriza_controlador_ajax`), a `acao_ajax` da operação, um
  `infra_hash`, pares na ordem do `prepararExecucao` servido, efeito
  obrigatoriamente `ConsultaSemEfeitoObservado`.
- A tela é conferida por lista fechada; o órgão é o efetivo do navegador
  (a última opção `selected`, "Todos" nas 37 telas); as duas formas do
  `prepararExecucao` são linhas exatas.
- O destino é o único item cuja sigla (antes de ` - `) é a pedida, sem
  maiúsculas ASCII; nenhum ou vários são recusa com a lista servida. O
  processo já aberto no destino é recusa (o reenvio não muda `aberto_em`:
  a leitura não provaria nada).
- O POST leva só o observado: ocultos como servidos, `hdnUnidades =
  ID±DESCRIÇÃO`, `selOrgao`, os textos vazios e, com `--manter-aberto`,
  `chkSinManterAberto=on` (a variante gravada nos 25 envios). Sem
  `--manter-aberto` a caixa vai como servida; esse efeito não está
  gravado e é o que o live deve medir primeiro.
- Resposta esperada: o 302 para `arvore_visualizar` com
  `acao_origem=procedimento_enviar` (27 de 27); outra é `falha_do_envio`,
  nunca repetida; a reconciliação segue a tabela do
  [contrato](../../contracts/cli/processo-enviar.md).

Descartado: escolher por prefixo ou pelo nome (a busca da UFRJ devolve
`ADCENTRAL` e suas subunidades); vários destinos e grupos (gravado só no
PEN1); retorno programado, remover anotação e e-mail (fora do escopo).

## Busca do destino no GO 5.0.3

**O que as gravações mostram.** As 4 telas de envio do GO servem o
`selOrgao` com 64 opções (`null`, "Todos" = `""`, ambas `selected`, e 62
órgãos com rótulo único, `ABC` = `28`); rótulos com espaço (`CASA CIVIL`,
`GOIAS TURISMO`) que a sigla escreve sem ele (`CASACIVIL/…`). Na cassette
`grupo-envio-ciclo-go` (consulta da mesma `acao_ajax`, tela de grupo),
duas vezes uma resposta vazia seguida de 39 itens `…/GECONT-…`, sem
nenhuma `ABC/…`. Nenhuma resposta gravada do GO tem 50 itens, e o corpo das
consultas não foi gravado: o termo e o órgão de cada uma vêm do Todos
(`termos_da_unidade`, `orgao_da_sigla`; testes `test_envio_010.py`,
medido em 20/10/2026), que também é a única fonte do **corte em 50** com
"Todos". O [replay da busca](../validacao-replay/2026-11-04-processo-enviar-acervo.md)
conta isso.

**Alternativas.**

1. **Até três consultas: órgão do prefixo com a sigla, órgão da tela com
   a sigla, órgão da tela com o termo alargado** (escolhida).
2. **A ordem do Todos, até quatro**: cada órgão com os dois termos.
3. **Só o termo alargado**: uma consulta a menos quando a sigla inteira
   não casa, mas troca o termo também onde a busca inteira funciona (UFRJ:
   `ADCENTRAL/GR` casa inteira).
4. **Lupa de unidades por órgão**: lista paginada sem gravação de uso.
5. **Consulta única** (antes): não acha no GO o que o Todos acha.

| Critério | 1 | 2 | 3 | 5 |
| --- | --- | --- | --- | --- |
| Consultas, pior caso | 3 | 4 | 2 | 1 |
| Sem `/`, como antes | sim | sim | sim | sim |
| Cobre o corte em 50 (Todos) | sim | sim | não | não |
| Cobre a quebra na `/` (gravada) | sim, no órgão da tela | sim, nos dois | sim | não |

**Decisão.** A ordem é a do Todos sem a quarta consulta (o termo
alargado no órgão do prefixo): o filtro existe pelo corte da lista longa,
e o termo alargado já devolve, no órgão da tela, a lista gravada (39
itens). A primeira consulta usa o órgão do prefixo só quando ele é outro
que o da tela; nenhum item exato ali leva à segunda; vários exatos
recusam (não há o que escolher no outro órgão). A terceira só sai quando
a segunda vem vazia, e dela vale só a sigla exata, nunca um parcial único
(medido no GO: `GECONT GERAL` devolve uma unidade de outro órgão). Uma
falha de consulta encerra a busca. `oferecidos` é a última resposta lida:
é a que o comando teve para decidir. A regra do rótulo não remove espaços
(como o Todos): `CASACIVIL/…` não escolhe `CASA CIVIL` e cai na segunda
consulta.

**Trade-off medido ao vivo.** Na prévia do GO para `ABC/GECONT-18141`
(o caso do Todos), as três consultas saíram e nenhuma trouxe a unidade:
o órgão `28` com a sigla inteira veio vazio, e o termo alargado devolveu
os mesmos 39 itens gravados
([relatório](../validacao-live/2026-11-04-processo-enviar-go.md)). Se a
unidade existe como destino, só a quarta consulta da alternativa 2 (ou a
lupa) poderia achá-la; isso não foi medido.

**`Location` do 302.** O Todos mediu no GO o `Location` de escritas em
`http://` no mesmo host (issue #766). As cassettes gravam a forma
relativa nos 3 envios e nos 32 `procedimento_concluir` do GO; `http://`
absoluto só no `infra_trocar_unidade` e no `procedimento_controlar`. O
`Location` segue dado, nunca seguido: relativo, ou `http(s)://` no host e
na pasta do POST (`consulta_do_controlador`, compartilhado com
`processo concluir`); outro host, porta ou pasta é
`redirecionamento_nao_reconhecido`.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-enviar.md) e
  [evidência da operação](../../contracts/evidencias/processo-enviar.md)
- [Observação estrutural](../como-o-sei-funciona/envio-do-processo.md)
- [Superfície da CLI](../../tests/cli_processo_enviar.rs)
- [Prévia e recusas](../../src/processo_enviar/tests_fluxo.rs),
  [escrita e reconciliação](../../src/processo_enviar/tests_reconciliacao.rs),
  [tela](../../src/processo_enviar/tests_tela.rs),
  [consulta e destino](../../src/processo_enviar/tests_destino.rs),
  [busca do destino no GO](../../src/processo_enviar/tests_busca.rs),
  [consulta AJAX](../../src/formulario_web/tests_ajax.rs)
- [Replay do acervo](../validacao-replay/2026-11-04-processo-enviar-acervo.md)
  e [relatório live](../validacao-live/2026-11-04-processo-enviar.md)

## Limitações e revisão

O corpo dos POST não foi gravado: `hdnUnidades` vem do script e do Todos.
O envio sem manter aberto, a recusa do SEI (HTML 200 com alerta) e um
destino de outro órgão não foram gravados. Escrita live **NOT_RUN**:
desfazer um envio exige sessão na unidade de destino (plano no relatório
live). Revisar se o ensaio mostrar: resposta diferente do 302, a origem
saindo também com manter aberto, `aberto_em` só atualizado depois de
outra navegação, ou a quarta consulta (termo alargado no órgão do
prefixo) medida e achando o destino que a busca atual não acha no GO.
