---
type: Contract
title: Propriedades do processo pela barra da árvore
description: ies processo NUP lê tipo, especificação, assuntos, interessados, observações, nível de acesso e data de geração pelo formulário procedimento_alterar (ou procedimento_consultar) publicado na barra do nó do processo, sem nunca enviá-lo
---

# Propriedades de `ies processo NUP` (Refs #59)

Complementa o [contrato da árvore](processo-obter-arvore.md). Depois da
árvore conferida (e dos andamentos), o comando lê as propriedades do
processo. Estrutura em
[propriedades do processo](../../docs/como-o-sei-funciona/propriedades-do-processo.md);
decisão em [processo.obter](../../docs/decisoes/processo-obter.md).

## Link na barra do processo

A barra do nó `PROCESSO` é a linha inteira `Nos[i].acoes = '<literal>';`
(o mesmo leitor de literais da árvore; `i` é o índice do nó `PROCESSO`).
As tags `<a>` do literal são lidas pelo tokenizador de atributos; o
`href` (entidades decodificadas) que começa por `controlador.php?` e tem
`acao=procedimento_alterar` é o link; sem ele, o de
`acao=procedimento_consultar` (observado na UnB quando o processo está
aberto só noutra unidade; o Todos mediu o mesmo na UFRJ 4.1 com o
processo fechado). O link é conferido como os outros links assinados:
relativo, uma `acao`, `id_procedimento` igual ao da árvore, um
`infra_hash` de 64 hex, chaves canônicas, resolvido contra a instalação.

**Link não publicado.** Barra ausente, vazia ou sem nenhum dos dois links
faz o comando falhar com `PropriedadesNaoPublicadasNaBarra` (saída 1, sem
GET, nada publicado). Só `--sem-propriedades` dispensa a
leitura. Link presente mas inválido ou repetido é recusa tipada.

## A tela

Um GET do link (+1 requisição). O formulário `form#frmProcedimentoCadastro`
**nunca** é enviado: nenhum POST, `btnSalvar` (`type=button`, escrita)
nunca participa. Leitura pelo tokenizador de atributos e pela árvore do
`tl`, sem regex e sem JavaScript:

- **Identidade.** `hdnProtocoloFormatado` igual ao NUP pedido e
  `hdnIdProcedimento` igual ao id da árvore (comparação opaca, byte a
  byte); um de cada, dentro do único `form#frmProcedimentoCadastro`.
- **`tipo`**: o texto da única `option` marcada de `selTipoProcedimento`;
  nenhuma marcada (ou a `option` `null`) é `TipoDoProcessoNaoMarcado`;
  `hdnIdTipoProcedimento`, se presente, tem de ser igual ao `value` dela.
- **`especificacao`**: `value` de `txtDescricao`.
- **`assuntos`**: cada `option` de `selAssuntos`, na ordem servida, com
  `value` decimal; o texto `<código> - <descrição>` vira
  `{"codigo", "descricao"}` quando o código só tem dígitos e pontos.
  Texto sem código nesse formato é `CodigoDoAssuntoNaoInterpretado`.
- **`interessados`**: o texto de cada `option` de
  `selInteressadosProcedimento`, na ordem servida (`[]` sem opção).
- **`observacoes`**: o texto de `txaObservacoes`.
- **`nivel_de_acesso`**: o único `rdoNivelAcesso` marcado, pelo `value`
  (`0` `publico`, `1` `restrito`, `2` `sigiloso`), com o `id` observado
  (`optPublico`, `optRestrito`, `optSigiloso`).
- **`hipotese_legal`**: o texto da única `option` marcada de
  `selHipoteseLegal` com `value` diferente de `null`; nenhuma: `null`.
  É a única ausência de saída que vem do domínio, não de leitura: a
  hipótese só se marca em nível restrito ou sigiloso (a confirmar pelo dono).
- **`data_de_geracao`**: `hdnDtaGeracao` na forma `dd/mm/aaaa`; ausente
  ou vazio é `DataDeGeracaoAusente`.

`hdnAssuntos` e `hdnInteressadosProcedimento` vêm **vazios** no HTML
servido (o `infraLupaSelect` os preenche no navegador): a fonte é o
`select`. Lista oculta servida não vazia é recusa
(`ListaOcultaDasPropriedadesPreenchida`), não palpite.

## Índice local

O link lido é guardado (`acao` `procedimento_alterar` ou
`procedimento_consultar`, a da barra; objeto = id). No **caminho 1**
(árvore pelo link guardado) o guardado da mesma `acao` é tentado
primeiro; login, redirect, página sem o formulário ou de outro processo o
apagam, e o link da árvore recém-lida é usado. Status de falha é erro de
transporte, sem nova tentativa.

## Orçamento

| Caminho | Padrão | `--sem-propriedades` | `--sem-andamentos` | os dois |
| --- | --- | --- | --- | --- |
| 1. guardados (árvore, histórico, propriedades) | 3 | 2 | 2 | 1 |
| 1. propriedades guardadas velhas | 4 | 2 | 3 | 1 |
| 4a. linha do Controle | 5 | 4 | 4 | 3 |

Nos outros caminhos, +1 sobre o [contrato dos andamentos](processo-obter-andamentos.md).
Sem link na barra, o comando falha antes de qualquer GET das propriedades.

## Saída

```json
"propriedades": {"tipo": "texto", "especificacao": "texto",
  "interessados": ["texto"],
  "assuntos": [{"codigo": "999.99", "descricao": "texto"}],
  "observacoes": "texto", "nivel_de_acesso": "publico",
  "data_de_geracao": "dd/mm/aaaa",
  "fonte": "procedimento_alterar"}
```

Lidas, `"propriedades"` sai de `nao_lidos`. `--sem-propriedades`: nenhum
pedido; a chave `propriedades` some, `"propriedades"` fica em
`nao_lidos` e o motivo é `"omitidas_por_sem_propriedades"`. Sem a flag,
o objeto é sempre o lido, ou o comando falha. Interessados
são nomes de pessoas e entidades: conteúdo pedido pelo usuário, publicado
na saída e nunca em log. Ids de assunto e de contato não são publicados.

## Efeitos

GET de formulário de edição: declarado **leitura** (nenhuma mudança de
estado observada; o Todos o usa como leitura). Nada a restaurar.

## Falhas (saída 1)

Link: `barra_do_processo_fora_da_forma_observada`,
`barras_do_processo_repetidas_na_arvore`,
`LinksDasPropriedadesRepetidosNaBarra`,
`EnderecoDasPropriedadesNaoReconhecido`,
`EnderecoDasPropriedadesDeOutroProcesso`.

Tela: `html_das_propriedades_nao_interpretado`,
`FormularioDasPropriedadesAusente`,
`formularios_das_propriedades_repetidos`,
`campo_das_propriedades_ausente`, `campo_das_propriedades_repetido`,
`NumeroDasPropriedadesDiferenteDoSolicitado`,
`IdDasPropriedadesDiferenteDoAberto`,
`TipoDoProcessoInconsistente`, `TipoDoProcessoNaoMarcado`,
`opcoes_marcadas_repetidas`,
`assunto_fora_da_forma_observada`, `CodigoDoAssuntoNaoInterpretado`,
`interessado_fora_da_forma_observada`,
`ListaOcultaDasPropriedadesPreenchida`,
`NivelDeAcessoNaoMarcado`,
`nivel_de_acesso_fora_da_forma_observada`,
`DataDeGeracaoAusente`, `data_de_geracao_fora_da_forma_observada`,
`TextoDasPropriedadesNaoDecodificado`.

Barra: `PropriedadesNaoPublicadasNaBarra` (sem `procedimento_alterar` nem
`procedimento_consultar`).

Login fora de palpite: `SessaoEncerrada`; 302 de outra
forma: `redirecionamento_nao_reconhecido`; status HTTP de falha, o erro
de transporte com `status_http`, sem retry.

Validação live: **PASS** na UnB, na PEN1 e em GO ([varredura UnB e GO](../../docs/validacao-live/2026-11-04-varredura-unb-go.md), [PEN1](../../docs/validacao-live/2026-11-04-processo-obter-movidos.md), [relatório](../../docs/validacao-live/2026-11-04-processo-obter-propriedades.md)).
