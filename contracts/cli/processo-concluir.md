---
type: CommandContract
title: Concluir o processo na unidade da sessão
operation: processo.concluir
command: ies processo concluir NUP [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo concluir NUP [--confirmar]` (Refs #172)

Conclui o processo **na unidade da sessão**, uma vez e por pedido
explícito, pela ação que a barra do nó `PROCESSO` da árvore publica. É a
primeira escrita de estado do processo. Estrutura observada em
[conclusão do processo](../../docs/como-o-sei-funciona/conclusao-do-processo.md);
a verificação é a situação `aberto_em` que
[`ies processo NUP`](processo-obter-situacao.md) já lê.

Segue as invariantes de escrita do [contrato da CLI](../cli.md): prévia,
confirmação e verificação; escrita não confirmada nunca se repete; saídas 0
sucesso, 1 falha, 2 uso inválido, 3 escrita não confirmada. Só a Web, a mesma
sessão; nenhuma URL montada (só as publicadas pela barra e pela tela do nó).

## Entrada

- `NUP`: o mesmo de `ies processo NUP`; inválido sai com 2
  (erro do clap) antes de qualquer arquivo ou requisição.
- `--unidade SIGLA` (global): conferida na tela do processo, como na leitura.
- `--confirmar`: conclui. Sem ele, só prévia.

## Comportamento

1. **Abre o processo** pelo caminho de [`ies processo NUP`](processo-obter-arvore.md),
   sem andamentos, propriedades nem marcadores, **pulando o link guardado
   da árvore** (passo 1): ele não publica a unidade, e a conclusão vale só
   para a unidade da sessão (`lnkInfraUnidade` da tela do processo).
2. **Lê a situação** (`aberto_em`) e a **ação da barra**: o link
   `procedimento_concluir` assinado (forma `tela`, SEI 4.1 e 5.0) ou o
   `onclick="concluirProcesso();"` (forma `funcao`, SEI 4.0), com o `href`
   assinado do nó raiz (`arvore_visualizar`) para achar a URL.
3. **Decide antes de escrever**, nesta ordem:

| Situação lida | Barra | Resultado |
| --- | --- | --- |
| não lida (sobrestado, anexado, fora da forma) | qualquer | saída 1, `SituacaoDoProcessoNaoLidaAntesDeConcluir` (`motivo`) |
| sem a unidade da sessão | sem concluir | saída 0, `alterado: false` (já não está aberto nela), sem escrita |
| sem a unidade da sessão | com concluir | saída 1, `UnidadeDaSessaoAusenteDaSituacao` |
| com a unidade da sessão | sem concluir | saída 1, `ConclusaoNaoPublicadaNaBarra` |
| com a unidade da sessão | barra ou endereço recusado | saída 1, subcódigo da barra |
| com a unidade da sessão | `tela` ou `funcao` | prévia (saída 0) ou escrita |

4. **Sem `--confirmar`**: publica `unidade`, `aberto_em` (siglas),
   `forma`, `depois_esperado` (as siglas sem a da sessão) e o plano de
   requisições. **Nenhuma** requisição além da abertura: a tela de
   conclusão (`tela`) e a tela do nó (`funcao`) não são pedidas.
5. **Com `--confirmar`**, uma escrita e nenhum retry:
   - `tela`: GET do link da barra; a resposta precisa ser HTML 200 com
     `form#frmDesentranharDocumento` na forma observada (abaixo), senão
     saída 1 sem POST. Um POST derivado pelo primitivo de formulário
     (`CONCLUIR_PROCESSO`).
   - `funcao`: GET do `arvore_visualizar` da raiz; a resposta precisa ser
     HTML 200 com `function concluirProcesso(){` na forma de três linhas,
     senão saída 1 sem escrita. **Um GET** da URL do corpo da função: esse
     GET é a escrita.
6. **Verifica** por uma leitura nova, a de `ies processo NUP` (o passo mais
   barato que o índice permitir, em geral o link da árvore: 1 GET):
   sucesso só se a situação lida não lista mais a unidade da sessão.

## Conferência da tela "Conclusão de Processo"

Antes do POST, todo controle enviável do formulário precisa ser um destes,
senão `campo_nao_previsto_na_tela_de_conclusao`: ocultos
`hdnInfraTipoPagina`, `hdnIdProtocolos`, `hdnIdProcedimentoDestino`;
`sbmSalvar`; `selProcedimentos` (`multiple`, nenhuma opção `selected`);
rádios `rdoConcluir` e `rdoPrazoReaberturaProgramada`; textos
`txtPrazoReaberturaProgramada` e `txtDiasReaberturaProgramada`; a caixa
`chkSinDiasUteisReaberturaProgramada`. Botões `button`/`reset` e
desabilitados não contam. Além disso:

| Fato | Variante ou `onde` (stderr) |
| --- | --- |
| Sem o formulário | `formulario_de_conclusao_ausente` |
| `hdnIdProtocolos` diferente do id aberto | `TelaDeConclusaoDeOutroProcesso` |
| Rádio `rdoConcluir` `S` ausente ou não `checked` | `OpcaoSomenteConcluirNaoMarcada` |
| Opção do processo `selected`, rádio de prazo ou caixa marcados, texto não vazio | `TelaDeConclusaoComReaberturaPreenchida` |

O POST leva, na ordem do documento: `hdnInfraTipoPagina` e `sbmSalvar`
como servidos, `rdoConcluir=S`, `txtPrazoReaberturaProgramada=`,
`txtDiasReaberturaProgramada=`, `hdnIdProtocolos` e
`hdnIdProcedimentoDestino` como servidos. O `action` é o servido
(conferido pelo primitivo). A janela não publica `lnkInfraUnidade`: a
unidade é a conferida na tela do processo
(`ConferenciaDaUnidade::NaTelaDeOrigemDoLink`).

## Resposta e escrita não confirmada

**Esperada**: na forma `tela`, 302 (não seguido) com `Location`
`controlador.php?acao=arvore_visualizar…` ou `controlador.php?acao=principal…`,
com `acao_origem=procedimento_concluir` e o `id_procedimento` aberto (o
`Location` é dado, nunca seguido: relativo, como gravado, ou `http(s)://`
no host e na pasta do POST, a forma que o Todos mediu no GO; outro host,
porta ou pasta não é o esperado); na
forma `funcao`, HTML 200 sem tela de login nem validação da Infra.
Qualquer outra (rede, tempo, status, 302 para outro destino, HTML 200 na
forma `tela`, login) é `falha_do_envio` e **não** repete a escrita: a
leitura de verificação reconcilia, uma vez.

| Resposta | Situação lida depois | Saída | Resultado |
| --- | --- | --- | --- |
| Esperada | sem a unidade da sessão | 0 | `alterado: true` |
| Não esperada | sem a unidade da sessão | 0 | `alterado: true`, `reconciliado_por_leitura: true`, `falha_do_envio` |
| Qualquer | com a unidade da sessão | 1 | `ConclusaoNaoAplicada` (com `falha_do_envio`, se houver) |
| Qualquer | leitura falhou ou situação não lida | 1 | `NaoConfirmadoNaConclusao` (campo `verificacao`: `VerificacaoPorLeitura`) |

Um SEI que processe a escrita depois da leitura não é detectável: antes de
repetir à mão, conferir com `ies processo NUP`. Falha **antes** da escrita
(abertura, barra, tela, cookie, derivação) não é escrita: saída 1.

## Efeito

`EfeitoAvaliado::AlteraEstadoDoProcesso { AndamentoAbertoNaUnidade }`:
fecha o andamento do processo na unidade da sessão; as outras unidades
continuam como estavam (observado). Desfaz-se com
[`ies processo reabrir`](processo-reabrir.md).

## Saída

```json
{"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
 "origem": "web", "confirmado": false, "alterado": false, "forma": "tela",
 "aberto_em": ["TESTE", "TESTE_1_1"], "depois_esperado": ["TESTE_1_1"],
 "depois": null, "reconciliado_por_leitura": false, "requisicoes": 3}
```

`forma` é `tela`, `funcao` ou `null` (nada a fazer). `depois` são as
siglas lidas na verificação. Nunca URL, `infra_hash` ou cookie. Falhas:
saída 1, stdout vazio e `Error: <variante de ErroConcluir>` em stderr;
`aberto_em`, `depois`, `falha_do_envio` e `verificacao` são campos das
variantes que os observaram.
A saída humana termina com "Para concluir, repita com --confirmar" na
prévia.

## Sequência e orçamento

Abertura = a de `ies processo NUP` sem extras, sem o passo 1: 3 GET a frio
pela linha do Controle (Controle, tela do processo, árvore), 2 GET pelo
link de trabalho guardado; uma pasta fechada soma 1. Verificação = a
leitura mais barata (1 GET pelo link da árvore que a abertura guardou).

| Caso | Forma `tela` | Forma `funcao` |
| --- | --- | --- |
| Uso inválido | 0 | 0 |
| Prévia, nada a fazer ou recusa antes da escrita | abertura | abertura |
| Tela recusada | abertura + 1 GET | abertura + 1 GET |
| `--confirmar` (qualquer resultado da verificação) | abertura + 1 GET + 1 POST + verificação | abertura + 2 GET + verificação |

A frio, com a árvore guardada pela abertura: 5 GET + 1 POST (`tela`) ou
6 GET (`funcao`). Os testes conferem a sequência exata.

## Fora do escopo

Reabrir (outro comando: [`processo reabrir`](processo-reabrir.md)), "Concluir e agendar reabertura" (`rdoConcluir=V`), concluir
vários processos (`selProcedimentos`), concluir noutra unidade sem trocar a
sessão.

## Evidência

[Evidência da operação](../evidencias/processo-concluir.md) (Todos, sete
instalações) e [observação estrutural](../../docs/como-o-sei-funciona/conclusao-do-processo.md);
[decisão](../../docs/decisoes/processo-concluir.md). Live: prévia e escrita
**PASS** na UFRJ 4.1.5 (forma `tela`) e na UnB 4.0.9 (forma `funcao`),
com as requisições do orçamento
([relatório](../../docs/validacao-live/2026-11-04-processo-concluir.md)).
