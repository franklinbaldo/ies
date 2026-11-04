---
type: CommandContract
title: Enviar o processo a uma unidade de destino
operation: processo.enviar
command: ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]` (Refs #173)

Envia (tramita) o processo da **unidade da sessão** a **uma** unidade de
destino, uma vez e por pedido explícito, pelo link "Enviar Processo" que a
barra do nó `PROCESSO` da árvore publica. Estrutura observada em
[envio do processo](../../docs/como-o-sei-funciona/envio-do-processo.md);
a verificação é a situação `aberto_em` que
[`ies processo NUP`](processo-obter-situacao.md) já lê.

Segue as invariantes de escrita do [contrato da CLI](../cli.md) e as peças
de [`processo concluir`](processo-concluir.md) (abertura, cookie, tela antes
da escrita, leitura de verificação). Saídas 0 sucesso, 1 falha, 2 uso
inválido, 3 efeito incerto. Só a Web, a mesma sessão; nenhuma URL montada:
o link da barra, o `action` da tela e a URL do autocomplete que o script
da tela declara.

## Entrada

- `NUP`: o de `ies processo NUP`; inválido sai com 2 (`entrada_invalida`)
  antes de qualquer arquivo ou requisição.
- `--para SIGLA`: **uma** sigla de unidade, como o autocomplete da tela a
  publica (`SIGLA - Nome`). Vazia, com caractere de controle ou com mais
  de 100 caracteres: 2 (`sigla_de_destino_invalida`); `--para` repetido:
  2 (`mais_de_um_destino_nao_suportado`). Antes de qualquer arquivo.
- `--manter-aberto`: marca "Manter processo aberto na unidade atual"
  (`chkSinManterAberto=on`). Sem ele, a caixa vai como servida (desmarcada).
- `--unidade SIGLA` (global): conferida na tela do processo.
- `--confirmar`: envia. Sem ele, só prévia.

## Comportamento

1. **Abre o processo** como [`processo concluir`](processo-concluir.md)
   (sem o link guardado da árvore, que não publica a unidade).
2. **Lê a situação** (`aberto_em`) e o **link da barra**
   (`<a href="controlador.php?acao=procedimento_enviar&…">`, conferido
   como endereço assinado do mesmo processo).
3. **Decide antes de pedir a tela**:

| Situação lida | Barra | Resultado |
| --- | --- | --- |
| não lida | qualquer | 1, `situacao_do_processo_nao_lida_antes_de_enviar` (`motivo`) |
| sem a unidade da sessão | sem enviar | 1, `processo_nao_aberto_na_unidade_da_sessao` |
| sem a unidade da sessão | com enviar | 1, `unidade_da_sessao_ausente_da_situacao` |
| com a unidade da sessão | sem enviar | 1, `envio_nao_publicado_na_barra` |
| com a unidade da sessão | link recusado | 1, subcódigo da barra |
| com a unidade da sessão | link | segue |

4. **GET da tela "Enviar Processo"** (o link da barra). Mostrar a tela não
   grava nada (observado: o histórico depois de duas telas e duas
   consultas só tem a criação e o "Processo remetido"). A tela é conferida
   (abaixo); fora da forma, 1 sem POST.
5. **Consulta do autocomplete** (`unidade_auto_completar_envio_processo`,
   POST ao `controlador_ajax.php`, URL do `infraAjaxAutoCompletar` da
   tela): o corpo do `prepararExecucao` servido, `palavras_pesquisa=SIGLA`,
   `id_orgao=` o órgão efetivo do `selOrgao` e, quando o script o traz,
   `unidade_atual=0`. É leitura (`ConsultaSemEfeitoObservado`). A resposta
   precisa ser XML 200 `<itens><item id descricao/>…</itens>`, senão 1.
6. **Escolhe o destino**: os itens cuja sigla (o texto de `descricao` antes
   de ` - `) é igual a `SIGLA`, sem distinguir maiúsculas ASCII. Nenhum: 1,
   `destino_nao_oferecido_pelo_sei` (`oferecidos`: as descrições servidas);
   mais de um: 1, `varios_destinos_com_a_sigla` (`candidatos`). Destino já
   em `aberto_em`: 1, `processo_ja_aberto_no_destino` (a leitura não
   distinguiria o efeito).
7. **Sem `--confirmar`**: publica `destino`, `aberto_em`, `depois_esperado`
   e o plano. Nenhum POST de escrita.
8. **Com `--confirmar`**, um POST e nenhum retry, derivado pelo primitivo
   de formulário (`ENVIAR_PROCESSO`).
9. **Verifica** por uma leitura nova (a de `ies processo NUP`, em geral
   1 GET pelo link da árvore guardado na abertura): o destino tem de estar
   em `aberto_em`; a unidade da sessão, só com `--manter-aberto`.

## Conferência da tela "Enviar Processo"

Formulário `frmAtividadeListar` (`method="post"`). Todo controle enviável
precisa ser um destes, senão `campo_nao_previsto_na_tela_de_envio`:
ocultos `hdnInfraTipoPagina`, `hdnIdProtocolos` (igual ao id aberto, senão
`tela_de_envio_de_outro_processo`), `hdnIdAtividades`, `hdnIdUnidade` e
`hdnUnidades`; `sbmEnviar`; `selProcedimentos`, `selOrgao`, `selUnidades`;
`txtUnidade`; caixas `chkSinManterAberto`, `chkSinRemoverAnotacoes`,
`chkSinEnviarEmailNotificacao`, `chkSinDiasUteis`,
`chkSinDiasUteisRetornoProgramado`, `chkSinDiasUteisReaberturaProgramada`;
rádios `rdoPrazo`, `rdoPrazoRetornoProgramado`,
`rdoPrazoReaberturaProgramada`; textos `txtPrazo`, `txtDias`,
`txtPrazoRetornoProgramado`, `txtDiasRetornoProgramado`,
`txtPrazoReaberturaProgramada`, `txtDiasReaberturaProgramada` (4.0 serve
`rdoPrazo`/`txtPrazo`/`txtDias`; 4.1 e 5.0, as de retorno e reabertura
programados).

| Fato | Subcódigo |
| --- | --- |
| Sem o formulário | `formulario_de_envio_ausente` |
| Caixa ou rádio marcado, texto ou `hdnUnidades`/`hdnIdUnidade` não vazio, opção em `selUnidades`, opção `selected` em `selProcedimentos` | `tela_de_envio_com_opcoes_preenchidas` |
| Órgão efetivo do `selOrgao` (a última opção `selected`, como o navegador) ausente ou `null` | `orgao_da_tela_de_envio_nao_selecionado` |
| Sem a linha `objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','…');` única | `autocompletar_de_unidade_nao_declarado` |
| `prepararExecucao` fora das duas formas observadas, ou repetido | `consulta_do_autocompletar_fora_da_forma_observada` |

O POST leva, na ordem do documento: os ocultos como servidos, salvo
`hdnUnidades` = `ID±DESCRIÇÃO` do item escolhido (a forma do
`infraLupaSelect`); `sbmEnviar`; `selOrgao` = o órgão efetivo;
`txtUnidade=`; `chkSinManterAberto=on` só com `--manter-aberto`; cada texto
servido, vazio. Nenhuma opção de `selProcedimentos`/`selUnidades`, nenhuma
outra caixa nem rádio. O `action` é o servido (conferido pelo primitivo);
a janela não publica `lnkInfraUnidade` (`NaTelaDeOrigemDoLink`).

## Resposta e efeito incerto

**Esperada**: 302 (não seguido) com `Location`
`controlador.php?acao=arvore_visualizar…`, `acao_origem=procedimento_enviar`
e o `id_procedimento` aberto (27 de 27 envios gravados). Qualquer outra
(rede, tempo, status, outro destino, HTML 200, login, validação da Infra)
é `falha_do_envio` e **não** repete a escrita.

| Resposta | Leitura depois | Saída | Resultado |
| --- | --- | --- | --- |
| Esperada | destino em `aberto_em` e unidade da sessão como pedido | 0 | `alterado: true` |
| Não esperada | idem | 0 | `alterado: true`, `reconciliado_por_leitura: true`, `falha_do_envio` |
| Qualquer | sem o destino | 1 | `escrita_nao_aplicada` / `envio_nao_aplicado` |
| Qualquer | destino, mas a unidade da sessão diferente do pedido | 1 | `escrita_diferente_da_solicitada` / `situacao_da_origem_diferente_da_pedida_apos_envio` |
| Qualquer | leitura falhou ou situação não lida | 3 | `efeito_incerto` / `efeito_incerto_no_envio` |

Falha antes do POST (abertura, barra, tela, consulta, destino, cookie,
derivação) não é escrita: saída 1. Antes de repetir à mão, conferir com
`ies processo NUP`.

## Efeito

`EfeitoAvaliado::AlteraEstadoDoProcesso { UnidadesComOProcessoAberto }`:
abre o processo na unidade de destino; sem `--manter-aberto`, o fecha na
da sessão. **Não se desfaz da sessão de origem**: sem manter aberto, o
processo sai da unidade; voltar exige uma sessão na unidade de destino
(enviar de volta ou concluir lá).

## Saída

```json
{"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
 "origem": "web", "confirmado": false, "alterado": false,
 "manter_aberto": false,
 "destino": {"sigla": "TESTE_1_1", "descricao": "TESTE_1_1 - Unidade 1.1", "id_unidade": "110000002"},
 "aberto_em": ["TESTE"], "depois_esperado": ["TESTE_1_1"], "depois": null,
 "reconciliado_por_leitura": false, "requisicoes": 5}
```

Nunca URL, `infra_hash` ou cookie. Falhas:
`{"codigo", "subcodigo", "erro", "retentavel": false}` mais `aberto_em`,
`oferecidos`, `candidatos`, `depois`, `falha_do_envio`, `verificacao` e
`status_http` quando houver. A saída humana da prévia termina com "Para
enviar, repita com --confirmar".

## Sequência e orçamento

Abertura = a de `processo concluir` (3 GET a frio pela linha do Controle,
2 pelo link de trabalho guardado). Verificação = 1 GET (link da árvore).

| Caso | Requisições |
| --- | --- |
| Uso inválido | 0 |
| Recusa pela situação ou pela barra | abertura |
| Tela recusada | abertura + 1 GET |
| Prévia, ou recusa da consulta ou do destino | abertura + 1 GET + 1 POST (consulta) |
| `--confirmar` (qualquer resultado da verificação) | abertura + 1 GET + 2 POST + verificação |

A frio: prévia 4 GET + 1 POST; `--confirmar` 5 GET + 2 POST. Os testes
conferem a sequência exata.

## Fora do escopo

Vários destinos e grupos de envio (`hdnUnidades` com `¥`), "Remover
anotação", "Enviar e-mail de notificação", retorno e reabertura
programados, a lupa de unidades, enviar vários processos.

## Evidência

[Evidência da operação](../evidencias/processo-enviar.md) (Todos, sete
instalações), [observação](../../docs/como-o-sei-funciona/envio-do-processo.md)
e [decisão](../../docs/decisoes/processo-enviar.md). Live: prévia **PASS** na UFRJ
4.1.5; escrita **NOT_RUN** (aguarda autorização,
[relatório](../../docs/validacao-live/2026-11-04-processo-enviar.md)).
