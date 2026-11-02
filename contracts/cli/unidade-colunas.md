---
type: Contract
title: Colunas do Nível de detalhe da unidade
description: ies unidade colunas lê, sem salvar, as 12 opções do Nível de detalhe da visão detalhada e quais estão marcadas
---

# `ies unidade colunas` (Refs #57)

Leitura da tela **Nível de detalhe** (`procedimento_configurar_detalhe`),
que define as colunas da [visão detalhada](unidade-listar-processos-visao-detalhada.md).
Faz parte da operação `unidade.listar_processos`: mesma sessão, mesma
tela de origem e mesmas travas. **Não salva** nada: `sbmSalvar` nunca é
enviado. Escrever a configuração é um comando futuro, com prévia e
confirmação.

## Modelo `ColunaDoDetalhe`

As 12 opções fixas observadas (PEN1 5.0.4 e GO 5.0.3, 02/11/2026):

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
             "cabecalho_tipo": "texto", "marcada": true}, ...]}
```

Sempre as 12, na ordem da tabela acima; `marcada` é a presença de
`checked` na caixa. Saída humana: uma linha `[x]`/`[ ]` por opção.

## Sequência e orçamento

O link está só na D (`a#ancNivelDetalhe`, `onclick="acaoConfigurarDetalhe('controlador.php?acao=procedimento_configurar_detalhe&…')"`);
a R não o publica (capturas de UnB, PEN1, GO).

| Tela no GET | Requisições |
| --- | --- |
| D | 1 GET (Controle) + 1 GET (Nível de detalhe) |
| R | 1 GET + POST D + 1 GET (Nível de detalhe) + POST R |

O link precisa ser relativo `controlador.php?…`, resolvido contra a URL da
tela D servida, na instalação autorizada, com `acao` única igual a
`procedimento_configurar_detalhe` e `infra_hash` único de 64 hexadecimais.
O GET segue as travas do transporte (sem redirect, 8 MB, tela de login).
Vindo da R, a volta à R segue a restauração da
[visão detalhada](unidade-listar-processos-visao-detalhada.md), com as
mesmas recusas `visao_do_controle_nao_restaurada` e campos.

| Fato | Subcódigo |
| --- | --- |
| D sem `ancNivelDetalhe` | `link_do_nivel_de_detalhe_ausente` |
| Mais de um `ancNivelDetalhe` | `links_do_nivel_de_detalhe_repetidos` |
| `onclick` fora da forma observada ou URL não autorizada | `link_do_nivel_de_detalhe_nao_reconhecido` |
| Formulário `frmAcessoExternoCancelar` ausente ou repetido | `formulario_do_nivel_de_detalhe_ausente`, `formularios_do_nivel_de_detalhe_repetidos` |
| `action` do formulário sem `acao=procedimento_configurar_detalhe` | `acao_do_formulario_do_nivel_de_detalhe_diferente` |
| Uma das 12 caixas ausente ou repetida | `opcao_do_nivel_de_detalhe_ausente`, `opcao_do_nivel_de_detalhe_repetida` |
| Caixa `chkSin*` fora das 12 | `opcao_do_nivel_de_detalhe_nao_prevista` |
| `label for=` ausente ou com outro texto | `rotulo_da_opcao_do_nivel_de_detalhe_diferente` |

## Evidência para a escrita futura (sem código nesta PR)

Ensaio do proprietário na PEN1 (02/11/2026, treinamento): o POST de salvar
envia só `hdnInfraTipoPagina` (como servido), `sbmSalvar` com o valor
servido e as caixas marcadas com valor `on` (sem atributo `value`); a
resposta é 200 sem redirect, uma página que fecha o modal; a mudança vale
imediatamente na D. A configuração original foi restaurada e conferida por
GET. Padrão desejado pelo proprietário: máximo de colunas, aplicado por
escrita explícita feita uma vez.

Ensaio offline com HTML sintético; live **NOT_RUN**.
