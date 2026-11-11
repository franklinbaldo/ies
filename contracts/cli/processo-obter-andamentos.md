---
type: Contract
title: Andamentos do processo pela árvore
description: ies processo NUP lê o histórico pelo link assinado da árvore, na visão servida ou completa, sem alegar completude além do lido
---

# Andamentos de `ies processo NUP` (Refs #59, fatia 2)

Complementa o [contrato da árvore](processo-obter-arvore.md). Depois da
árvore conferida, o comando lê o histórico do processo. Estrutura em
[histórico do processo](../../docs/como-o-sei-funciona/historico-do-processo.md);
decisão em [processo.obter](../../docs/decisoes/processo-obter.md).

## Link do histórico

A árvore publica o link assinado num `div#divConsultarAndamento`, numa de
duas formas, lidas por forma exata de texto (atributos pelo tokenizador
de `html_atributos`; sem regex, sem executar JavaScript):

- `onclick="consultarAndamento();"` e, no `script`, a linha
  `function consultarAndamento(){` seguida de
  `parent.document.getElementById('ifrVisualizacao').src = '<URL>';`
  (UnB 4.0.9);
- `onclick="consultarAndamento('<URL>');"` (GO 5.0.3).

A URL tem de ser relativa (`controlador.php?`), com uma
`acao=procedimento_consultar_historico`, `id_procedimento` igual ao da
árvore e um `infra_hash` de 64 hex, chaves canônicas (a mesma regra do
`src` de `ifrArvore`); é resolvida contra a instalação. As duas formas
juntas ou repetidas são `links_do_historico_repetidos_na_arvore`; um
`onclick` ou uma função fora dessas formas,
`link_do_historico_fora_da_forma_observada`.

**Link não publicado.** Em GO, duas árvores não publicaram o link em
nenhuma forma. Sem o link, o histórico não é lido: o comando falha com
`LinkDoHistorico(LinkDoHistoricoAusenteNaArvore)` (saída 1, nada é
publicado como `null`). Só `--sem-andamentos` dispensa a leitura, e
então o JSON sai sem a chave `andamentos`, com
`"andamentos_nao_lidos_motivo": "omitidos_por_sem_andamentos"`. Link
presente mas inválido ou repetido também é recusa tipada.

## Visões e requisições

- **Padrão**: 1 GET do link. Publica a visão que a página serviu, lida de
  `hdnTipoHistorico` no `form#frmProcedimentoHistorico` (`R` →
  `"resumido"`, `P` → `"completo"`; outro valor ou campo ausente/repetido é
  `TipoDoHistoricoNaoPublicado`; campo sem atributo `value` é
  `TipoDoHistoricoSemValor`). Normalmente `R`.
- **`--andamentos-completos`**: o GET e, se ele serviu `R`, o POST de
  `verHistorico('P')`: as três linhas servidas da função
  (`function verHistorico(valor){`,
  `document.getElementById('hdnTipoHistorico').value = valor;`,
  `document.getElementById('frmProcedimentoHistorico').submit();`) são
  conferidas literalmente antes de derivar (`RegraDeVerHistoricoAusenteNaPagina`):
  são o que o POST emula sem executar JS. A âncora `a#ancTipoHistorico` não é
  conferida; a visão servida depois do POST é (`VisaoDoHistoricoDiferenteDaPedida`).
  O POST vai para o `action`
  assinado com os ocultos como servidos, só `hdnTipoHistorico=P`
  reescrito, sem botão (`btnAtualizarAndamento`, escrita, é `type=button`
  e nunca participa). Só a visão `P` é publicada; resposta `R` é
  `VisaoDoHistoricoDiferenteDaPedida`. Se o GET já serviu `P`, não
  há POST.
- **`--sem-andamentos`**: nenhum pedido ao histórico; a chave
  `andamentos` some, `"andamentos"` fica em `nao_lidos` e o motivo é
  `"omitidos_por_sem_andamentos"`. Com `--andamentos-completos` sai 2
  (`sem_andamentos_incompativel_com_andamentos_completos`).

## Índice local

O link lido com sucesso é guardado (`acao`
`procedimento_consultar_historico`, objeto = id). No **caminho 1** (árvore
pelo link guardado) o link guardado do histórico é tentado primeiro;
login, redirect ou página de outro NUP o apagam, e o link da árvore
recém-lida é usado. Status de falha (por exemplo 503) não é palpite: é
erro de transporte, sem nova tentativa. Nos caminhos 2–4 usa-se o link da
árvore recém-lida.

## Orçamento

| Caminho | Padrão | `--andamentos-completos` | `--sem-andamentos` |
| --- | --- | --- | --- |
| 1. link da árvore, histórico guardado | 2 | 3 | 1 |
| 1. guardado que falha por palpite | 3 | 4 | 1 |
| 2. entrada por id | 4 | 5 | 3 |
| 3. pesquisa rápida guardada | 4 | 5 | 3 |
| 4a. linha do Controle | 4 | 5 | 3 |
| 4b. pesquisa rápida do Controle | 5 | 6 | 4 |

Sem link na árvore (e sem guardado), o histórico custa 0; se o GET já
servir `P`, o completo custa o mesmo que o padrão.

## Contagem e completude

A legenda da tabela é `Lista de Andamentos (N registro):` ou
`(N registros):`; outra forma (inclusive uma com faixa de paginação) é
`legenda_dos_andamentos_fora_da_forma_observada`, ausente é
`legenda_dos_andamentos_ausente`. Linhas além de N são
`AndamentosAlemDoTotalDeclarado`. Linhas aquém de N saem com
`total_declarado: N` e `faixa_observada: {"inicio": 1, "fim": linhas}`:
o IES nunca alega completude além do lido. Nenhum paginador foi visto
até 75 linhas ao vivo (GO). No [replay do acervo](../../docs/validacao-replay/2026-11-04-processo-obter-acervo.md)
(120 páginas, sete instalações, a maior com 57 linhas na UFRJ) a
`divInfraAreaPaginacaoSuperior` vem sempre vazia e as linhas igualam N.
Históricos acima de 75 linhas: **NOT_RUN**.

## Saída

```json
"andamentos": {"tipo": "resumido", "total_declarado": 3,
  "faixa_observada": {"inicio": 1, "fim": 3},
  "itens": [{"ocorrido_em": "dd/mm/aaaa hh:mm", "unidade": "SIGLA",
             "usuario": "sigla ou null", "descricao": "texto"}]}
```

`tipo` é o nome da visão do SEI ("Ver histórico completo"), não uma
alegação de completude. Itens na ordem servida, sem id inventado. O motivo
fica numa chave irmã, e não dentro do objeto. Com `--sem-andamentos` a
chave `andamentos` some; sem essa flag o objeto é sempre o lido, ou o
comando falha.

## Efeitos

O GET do histórico é leitura (lançamento de andamento: não medido). O
POST `P` é declarado `PaginaOuFiltroLembrado`: não foi medido se o
servidor lembra a visão na sessão. O IES não restaura a visão.

## Falhas (saída 1)

Link: `link_do_historico_fora_da_forma_observada`,
`links_do_historico_repetidos_na_arvore`,
`endereco_do_historico_nao_reconhecido`,
`EnderecoDoHistoricoDeOutroProcesso`,
`html_da_arvore_nao_interpretado`, e `LinkDoHistoricoAusenteNaArvore`
(a árvore não publica o link do histórico; só com andamentos pedidos).

Página: os do leitor do histórico, `src/historico.rs` (`titulo_ausente`,
`NumeroDoHistoricoDiferenteDoSolicitado`, `historico_ausente`,
`tabelas_do_historico_repetidas`, `linha_invalida`, `historico_vazio`…),
`legenda_dos_andamentos_ausente`,
`legenda_dos_andamentos_fora_da_forma_observada`,
`AndamentosAlemDoTotalDeclarado`, `TipoDoHistoricoNaoPublicado`,
`CharsetDoHistoricoNaoSuportado`.

Visão completa: `RegraDeVerHistoricoAusenteNaPagina`,
`VisaoDoHistoricoDiferenteDaPedida` e os do primitivo de formulário.
Login (200 ou 302) fora de palpite é `SessaoEncerrada`;
status HTTP de falha (503 do gateway visto em GO) sai com o erro de
transporte e `status_http`, sem retry.
