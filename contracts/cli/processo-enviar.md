---
type: CommandContract
title: Enviar o processo a uma unidade de destino
operation: processo.enviar
command: ies processo enviar NUP --para SIGLA [--manter-aberto] [--remover-anotacoes] [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo enviar NUP --para SIGLA [--manter-aberto] [--remover-anotacoes] [--confirmar]` (Refs #173, #333)

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
- `--remover-anotacoes`: na **prévia**, escolhe o checkbox real
  `chkSinRemoverAnotacoes`, quando habilitado na tela de envio. Se ausente,
  mas a barra oferecer `anotacao_registrar` e o formulário do mesmo processo
  for legível, planeja limpar `txaDescricao` antes do envio; sem ambos os
  caminhos, recusa com `remocao_de_anotacoes_nao_oferecida`.
  `--confirmar` com checkbox é admitido somente quando a tela de anotações
  da mesma origem foi oferecida e lida, permitindo releitura independente.
  A alternativa `limpar_anotacao` salva `txaDescricao=""` uma única vez
  se houver anotação, exige releitura independente do texto vazio **antes**
  de enviar, e só então tramita. Com texto já vazio, dispensa a gravação da
  anotação. A leitura de `aberto_em` comprova tramitação, não exclusão
  de anotações. Aprovado em treinamento e liberado em produção:
  [ensaio de 10/11/2026](../../docs/validacao-live/2026-11-10-processo-enviar-remover-anotacoes-unb.md).
  A prévia consulta o formulário `anotacao_registrar` da própria origem,
  quando o link é publicado na barra, e expõe `anotacao_antes` como
  `true` (texto presente), `false` (campo vazio) ou campo omitido (link
  não observado); essa primeira leitura **não prova a remoção posterior**.
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
4a. Com `--remover-anotacoes`, se a barra publicar o link assinado
   `anotacao_registrar` do mesmo processo, faz **um GET de leitura** do
   `frmAnotacaoCadastro`, confere o `hdnIdProtocolo` contra o processo
   aberto e lê a presença do texto de `txaDescricao`. Link não publicado
   não equivale a anotação ausente. Tela de outro processo, campos
   ambíguos ou GET recusado interrompem a prévia antes do autocomplete.
5. **Consulta do autocomplete** (`unidade_auto_completar_envio_processo`,
   POST ao `controlador_ajax.php`, URL do `infraAjaxAutoCompletar` da
   tela): o corpo do `prepararExecucao` servido, `palavras_pesquisa=TERMO`,
   `id_orgao=ÓRGÃO` e, quando o script o traz, `unidade_atual=0`. É
   leitura (`ConsultaSemEfeitoObservado`). Cada resposta precisa ser XML
   200 `<itens><item id descricao/>…</itens>`, senão 1 e a busca para (sem
   repetir nem seguir para a próxima consulta).
6. **Busca e escolha do destino**: em cada resposta valem só os itens
   cuja sigla (o texto de `descricao` antes de ` - `) é igual a `SIGLA`,
   sem distinguir maiúsculas ASCII; vale também para o termo alargado, que
   só amplia a busca. A ordem é a do Todos, no máximo quatro consultas,
   cada uma uma vez; a seguinte só sai se a anterior veio vazia ou sem a
   sigla exata. Um item exato decide; vários recusam.

| # | Quando | Termo | Órgão |
| --- | --- | --- | --- |
| 1 | `SIGLA` tem `/` e o prefixo antes da primeira `/` é o rótulo de uma única opção do `selOrgao` (espaços colapsados, sem caixa; fora `""` e `null`), diferente do órgão da tela | `SIGLA` | o `value` dessa opção |
| 2 | como 1, e há termo alargado | o último segmento sem o sufixo `-<dígitos>` (`GECONT` de `ABC/GECONT-18141`), se não vazio | o mesmo de 1 |
| 3 | sempre | `SIGLA` | o efetivo do `selOrgao` |
| 4 | `SIGLA` tem `/` e há termo alargado | o termo alargado | o efetivo |

   Nenhum item exato: 1, `destino_nao_oferecido_pelo_sei` (`oferecidos`:
   as descrições da **última** resposta lida); mais de um: 1,
   `varios_destinos_com_a_sigla` (`candidatos`). Sem `/` só há a consulta
   3, como antes. Destino já em `aberto_em`: 1,
   `processo_ja_aberto_no_destino` (a leitura não distinguiria o efeito).
7. **Sem `--confirmar`**: publica `destino`, `aberto_em`, `depois_esperado`
   e o plano. Com `--remover-anotacoes`, publica `remover_anotacoes: true`
   e `metodo_remocao: "checkbox_no_envio" | "limpar_anotacao"`, conforme
   os controles observados. Sem a flag, não altera a saída legada.
   Quando a anotação foi lida, `anotacao_antes: true/false` aparece na
   prévia; se o link não foi oferecido, o campo é omitido.
   Nenhum POST de escrita.
8. **Com `--confirmar`**, o método `checkbox_no_envio` só prossegue
   quando a origem oferece a tela `anotacao_registrar`, previamente lida.
   Um único POST de envio com `chkSinRemoverAnotacoes=on`, sem retry,
   é derivado pelo formulário (`ENVIAR_PROCESSO`). No método
   `limpar_anotacao`, se `txaDescricao` possui texto e a mesma tela oferece
   `sbmRegistrarAnotacao`, um POST de salvamento com descrição vazia
   é derivado do formulário, seguido por GET independente da anotação.
   Falha em comprovar o texto vazio interrompe antes do envio; não há
   repetição da limpeza. Se o campo já estava vazio, não há POST de
   salvamento. Após comprovar ausência, há **um único POST de envio**.
9. **Verifica** por uma leitura nova (a de `ies processo NUP`, em geral
   1 GET pelo link da árvore guardado na abertura): o destino tem de estar
   em `aberto_em`; a unidade da sessão, só com `--manter-aberto`.
   Com checkbox, outro GET de `anotacao_registrar` exige `txaDescricao`
   vazio e a mesma identidade de processo. Presença de texto após o
   envio é alteração parcial (saída 1); releitura indisponível ou
   inconclusiva é efeito incerto (saída 3), sem retry. No fallback,
   a limpeza e a releitura acontecem antes do envio: anotação ainda
   preenchida recusa o envio (1); leitura inconclusiva da limpeza recusa
   o envio com saída 3. O envio posterior continua sujeito à
   reconciliação do destino já existente.

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

O POST **sem `--remover-anotacoes`** leva, na ordem do documento: os ocultos como servidos, salvo
`hdnUnidades` = `ID±DESCRIÇÃO` do item escolhido (a forma do
`infraLupaSelect`); `sbmEnviar`; `selOrgao` = o órgão efetivo;
`txtUnidade=`; `chkSinManterAberto=on` só com `--manter-aberto`; cada texto
servido, vazio. Nenhuma opção de `selProcedimentos`/`selUnidades`, nenhuma
outra caixa nem rádio. O `action` é o servido (conferido pelo primitivo);
a janela não publica `lnkInfraUnidade` (`NaTelaDeOrigemDoLink`).

## Leitura independente da anotação na origem (#333)

Só na prévia com `--remover-anotacoes`, se a barra servir um link
`anotacao_registrar` para o mesmo `id_procedimento`, o IES consulta a tela
`frmAnotacaoCadastro` sem escrever. O campo oculto `hdnIdProtocolo`
identifica o processo; `txaDescricao` com texto não vazio indica anotação
presente na unidade de origem. A tela só permite comprovar o **antes**.
A prévia publica `anotacao_antes` sem converter ausência de observação em
ausência de anotação. Leitura recusada é falha tipada, antes do autocomplete.

| Forma não observada | Subcódigo |
| --- | --- |
| Barra ilegível | `barra_de_anotacoes_nao_interpretada` |
| Endereço de anotação inválido | `endereco_de_anotacoes_nao_reconhecido` |
| Links de anotação repetidos | `links_de_anotacoes_repetidos` |
| HTML de anotação ilegível | `tela_de_anotacoes_nao_interpretada` |
| Formulário ausente | `formulario_de_anotacoes_ausente` |
| Campo não publicado | `anotacao_nao_publicada` |
| Identidade divergente | `anotacao_de_outro_processo` |
| Campos repetidos | `campos_de_anotacao_repetidos` |

## Resposta e efeito incerto

**Esperada**: 302 (não seguido) com `Location`
`controlador.php?acao=arvore_visualizar…`, `acao_origem=procedimento_enviar`
e o `id_procedimento` aberto (27 de 27 envios gravados, todos relativos).
O `Location` é dado: relativo, ou `http(s)://` no host e na pasta do POST
(o Todos mediu `http://` no mesmo host no GO); outro host, porta ou pasta
não é o esperado. Qualquer outra
(rede, tempo, status, outro destino, HTML 200, login, validação da Infra)
é `falha_do_envio` e **não** repete a escrita.

| Resposta | Leitura depois | Saída | Resultado |
| --- | --- | --- | --- |
| Esperada | destino em `aberto_em` e unidade da sessão como pedido | 0 | `alterado: true` |
| Não esperada | idem | 0 | `alterado: true`, `reconciliado_por_leitura: true`, `falha_do_envio` |
| Qualquer | sem o destino | 1 | `escrita_nao_aplicada` / `envio_nao_aplicado` |
| Qualquer | destino, mas a unidade da sessão diferente do pedido | 1 | `escrita_diferente_da_solicitada` / `situacao_da_origem_diferente_da_pedida_apos_envio` |
| Qualquer | leitura falhou ou situação não lida | 3 | `efeito_incerto` / `efeito_incerto_no_envio` |

Quando o fallback já comprovou a limpeza e a releitura da tramitação falha,
continua sendo saída 3, mas a saída humana e o JSON preservam o fato
`anotacao_depois: false` (ausência relida antes de tentar enviar). Isso
**não** significa que a tramitação tenha sido comprovada.

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
| Prévia, ou recusa da consulta ou do destino | abertura + 1 GET + C POST (consultas); +1 GET se anotação publicada e a flag pedida |
| `--confirmar` (qualquer resultado da verificação) | abertura + 1 GET + C POST + 1 POST + verificação; com remoção via checkbox, +2 GET da anotação (antes/depois) |
| `--confirmar --remover-anotacoes` com fallback | mesma base, +1 GET da anotação antes; se há texto, +1 POST de limpeza e +1 GET de conferência **antes** do envio. Sem texto, nenhum POST de limpeza |

C, as consultas da busca (passo 6):

| `SIGLA` | C |
| --- | --- |
| sem `/` | 1 |
| com `/`, sem órgão do prefixo | 1 (a consulta 3 decide) ou 2 (mais a 4) |
| com `/` e órgão do prefixo | 1 a 4, a primeira com a sigla exata (ou vários) encerra |

A frio e sem `/`, sem leitura extra: prévia 4 GET + 1 POST; `--confirmar` 5 GET + 2 POST. Os
testes conferem a sequência exata de cada caminho.

## Fora do escopo

Vários destinos e grupos de envio (`hdnUnidades` com `¥`),
"Enviar e-mail
de notificação", retorno e reabertura
programados, a lupa de unidades, enviar vários processos.

## Evidência

[Evidência da operação](../evidencias/processo-enviar.md) (Todos, sete
instalações), [observação](../../docs/como-o-sei-funciona/envio-do-processo.md)
e [decisão](../../docs/decisoes/processo-enviar.md). Live: prévia **PASS** na UFRJ
4.1.5; GO 5.0.3: `CBM/GECONT-18149` e `DETRAN/GECONT-18166` achados na consulta 1
(**PASS**), `ABC/GECONT-18141`, inexistente, recusado (**PASS**,
[relatório](../../docs/validacao-live/2026-11-04-processo-enviar-go.md)); escrita **PASS** na UFRJ 4.1.5 em 04/11/2026: com `--manter-aberto` o processo fica
aberto no destino e na origem; sem ele, só no destino (a caixa desmarcada tira o
processo da origem), 6 requisições cada
([relatório](../../docs/validacao-live/2026-11-04-processo-enviar-escrita.md)); escrita **PASS** na UnB
4.0.9 (Unidade1 → Unidade2) e no PEN1 5.0.4 (TESTE → TESTE_1_1) em 04/11/2026, o mesmo
padrão, prévia a frio 5 e `--confirmar` 6 requisições
([relatório](../../docs/validacao-live/2026-11-05-processo-enviar-escrita-unb-pen1.md)).
