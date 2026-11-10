---
type: Contract
title: Colunas do Nível de detalhe da unidade
description: ies unidade colunas lê, sem salvar, as opções do Nível de detalhe da visão detalhada, quais a instalação publica e quais estão marcadas
---

# `ies unidade colunas` (Refs #57)

Leitura da tela **Nível de detalhe** (`procedimento_configurar_detalhe`),
que define as colunas da [visão detalhada](unidade-listar-processos-visao-detalhada.md).
Faz parte da operação `unidade.listar_processos`: mesma sessão, mesma
tela de origem e mesmas travas. **Não salva** nada: `sbmSalvar` nunca é
enviado. Escrever a configuração é a operação `unidade.definir_colunas`,
`ies unidade colunas --definir todas|LISTA [--confirmar]`, com prévia,
confirmação e verificação ([contrato da escrita](unidade-definir-colunas.md)).

## Modelo `ColunaDoDetalhe`

O **catálogo** são as 12 opções conhecidas, observadas na PEN1 5.0.4 e em
GO 5.0.3 (02/11/2026), que publicam todas. A tela de uma instalação pode
publicar um **subconjunto**: a UnB 4.0.9 publica 11, sem
`chkSinNivelPrioritarios` (Prioridade), observado no ensaio live de
02/11/2026, em que a versão anterior deste comando falhou com
`opcao_do_nivel_de_detalhe_ausente`.

| Chave | Campo | Rótulo da opção | Cabeçalho na D |
| --- | --- | --- | --- |
| `atribuicao` | `chkSinAtribuicao` | Atribuição | texto `Atribuição` |
| `anotacao` | `chkSinAnotacao` | Anotação | texto `Anotação` |
| `tipo` | `chkSinTipoProcesso` | Tipo do Processo | texto `Tipo` |
| `especificacao` | `chkSinEspecificacao` | Especificação | texto `Especificação` |
| `prioridade` | `chkSinNivelPrioritarios` | Prioridade | texto `Prioridade` |
| `interessados` | `chkSinInteressados` | Interessados | texto `Interessados` (itens) |
| `observacao` | `chkSinObservacao` | Observação | texto `Observação` |
| `controle_de_prazo` | `chkSinControlePrazo` | Controle de Prazo | só ícone, rótulo `Controle de Prazo` |
| `para_devolver` | `chkSinRetornoDevolver` | Para Devolver | só ícone, rótulo `Para Devolver` |
| `aguardando_retorno` | `chkSinRetornoAguardando` | Aguardando Retorno | só ícone, rótulo `Aguardando Retorno` |
| `ultima_movimentacao` | `chkSinUltimaMovimentacao` | Última Movimentação na Unidade | texto `Última Mov.` (`&nbsp;` no SEI) |
| `marcadores` | `chkSinMarcadores` | Marcadores | texto `Marcadores` |

Os três cabeçalhos de ícone não têm texto: o rótulo é o `aria-label` do
ícone (GO 5.0.3) ou o argumento de `infraTooltipMostrar` (UnB 4.0.9). O
proprietário registrou que, com as 12 marcadas na PEN1, a D mostrou 10
cabeçalhos de texto; as capturas estruturais de GO (12 marcadas) e da UnB
mostram, além deles, as três colunas de ícone. O modelo conta só as nove
de texto em `colunas_ausentes` e lê as de ícone quando aparecem.

## Saída

```json
{"unidade": "TESTE", "origem": "web",
 "opcoes": [{"coluna": "atribuicao", "campo": "chkSinAtribuicao",
             "rotulo": "Atribuição", "cabecalho": "Atribuição",
             "cabecalho_tipo": "texto", "disponivel": true,
             "marcada": true}, ...]}
```

Sempre as 12 do catálogo, na ordem da tabela acima. `disponivel` diz se a
tela publicou a caixa (escolha: um campo por opção, em vez de uma lista
`indisponiveis` à parte, para a forma de cada opção não variar); `marcada`
é a presença de `checked` na caixa, e é `false` quando `disponivel` é
`false`. Saída humana: uma linha `[x]`/`[ ]` por opção publicada e `[-]`,
com a indicação "indisponível nesta instalação", para as demais.

## Sequência e orçamento

O link está só na D (`a#ancNivelDetalhe`, `onclick="acaoConfigurarDetalhe('controlador.php?acao=procedimento_configurar_detalhe&…')"`);
a R não o publica (capturas de UnB, PEN1, GO).

| Tela no GET | Requisições |
| --- | --- |
| D | 1 GET (Controle) + 1 GET (Nível de detalhe) |
| R | 1 GET + POST D + 1 GET (Nível de detalhe); a sessão fica na D |

O link precisa ser relativo `controlador.php?…`, resolvido contra a URL da
tela D servida, na instalação autorizada, com `acao` única igual a
`procedimento_configurar_detalhe` e `infra_hash` único de 64 hexadecimais.
O GET segue as travas do transporte (sem redirect, 8 MB, tela de login).
Vindo da R, o comando **não volta** à R: a sessão fica na D, como em
[`listar-processos`](unidade-listar-processos-visao-detalhada.md) (decisão
do proprietário, 02/11/2026). Regime estável: 2 GET.

| Fato | Variante ou `onde` (stderr) |
| --- | --- |
| D sem `ancNivelDetalhe` | `link_do_nivel_de_detalhe_ausente` |
| Mais de um `ancNivelDetalhe` | `links_do_nivel_de_detalhe_repetidos` |
| `onclick` fora da forma observada ou URL não autorizada | `link_do_nivel_de_detalhe_nao_reconhecido` |
| Formulário `frmAcessoExternoCancelar` ausente ou repetido | `formulario_do_nivel_de_detalhe_ausente`, `formularios_do_nivel_de_detalhe_repetidos` |
| `action` do formulário sem `acao=procedimento_configurar_detalhe` | `AcaoDoFormularioDoNivelDeDetalheDiferente` |
| Nenhuma das 12 caixas publicada | `ErroNivelDeDetalhe::NenhumaOpcaoDoDetalhePublicada` |
| Uma caixa do catálogo publicada duas vezes | `ErroNivelDeDetalhe::OpcaoDoDetalheRepetida` |
| Caixa `chkSin*` fora do catálogo (falha fechada) | `opcao_do_nivel_de_detalhe_nao_prevista` |
| Caixa publicada com `label for=` ausente ou com outro texto | `ErroNivelDeDetalhe::RotuloDaOpcaoDoDetalheDiferente` |

Opção do catálogo **ausente** não é recusa: sai com `disponivel: false`.
`opcao_do_nivel_de_detalhe_ausente` foi aposentado na evolução de #57
para catálogo parcial; [decisão](../../docs/decisoes/unidade-listar-processos.md).

## Evidência para a escrita ([`--definir`](unidade-definir-colunas.md))

Ensaio do proprietário na PEN1 (02/11/2026, treinamento): o POST de salvar
envia só `hdnInfraTipoPagina` (como servido), `sbmSalvar` com o valor
servido e as caixas marcadas com valor `on` (sem atributo `value`); a
resposta é 200 sem redirect, uma página que fecha o modal; a mudança vale
imediatamente na D. A configuração original foi restaurada e conferida por
GET. Padrão desejado pelo proprietário: máximo de colunas, aplicado por
escrita explícita feita uma vez.

Ensaio offline com HTML sintético, inclusive uma tela de 11 opções na
estrutura da UnB 4.0.9. Live (02/11/2026): PEN1 PASS (3 marcadas); UnB
FAIL na versão anterior (Prioridade ausente), corrigido aqui, reensaio
**NOT_RUN** ([validação](../../docs/validacao-live/2026-11-02-leitura-controle.md)).
