---
type: CommandContract
title: Retornos programados da unidade no mes
operation: unidade.listar_retornos
command: ies unidade retornos
effect: read
implementation: partial
source_commit: 323eb013a1e7fac42c50336f838373a2cbe23e4b
---

# Retornos programados da unidade (Refs #253)

`ies unidade retornos [--mes AAAA-MM] [--json]` consulta a tela
**Retorno Programado** da unidade ativa do SEI Web. Sem `--mes`,
usa o mes publicado pela celula `diaHoje` do calendario do SEI,
nunca o relogio da maquina como resultado. Se a tela abriu no mes
visitado anteriormente e nao mostra `diaHoje`, a data UTC serve
somente para procurar o mes candidato pelos links assinados do
calendario; na virada UTC procura tambem o mes vizinho. So publica
o mes quando encontra uma unica celula `diaHoje` valida. `--mes` e validado antes de consultar
arquivos privados ou rede. `--unidade` exige identidade literal do
Controle, previamente reconhecido.

## Fonte e cobertura

A URL assinada `retorno_programado_listar` e obtida exclusivamente
do menu do Controle. A tela do calendario pode abrir no ultimo dia
visitado pela conta; o IES segue links GET publicados para o primeiro
dia do mes adjacente ate chegar ao solicitado (limite 24 saltos).
O link `Ver todo o mes`, com `data_inicial=01/MM/AAAA`
e `data_final=ultimo dia`, e a unica consulta da listagem.
Nao ha POST nem restauracao da tela.

As duas tabelas, quando presentes, distinguem a situacao pelo
`summary`: `aguardando` (outras unidades devem devolver) e
`devolver` (a unidade atual deve devolver). Le a coluna Processo,
Unidade, Data Programada, e opcionalmente Data de Retorno Efetiva e
Prazo Restante. A contagem de cada tabela deve conferir sua legenda. Quando so
`aguardando` aparece, a captura do mes inteiro pode omitir a tabela
`devolver`: esta forma so e aceita com uma unica
`divAguardandoAreaTabela` envolvendo a tabela reconhecida.
Essa moldura foi observada nas sete cassettes originais; uma tabela
solta, sem essa prova, e `lista_de_retornos_incompleta`.
A ausencia tambem pode ser demonstrada por uma area que publique
literalmente `Nenhum registro encontrado.`; nunca se inventam linhas.
Mes sem retornos so devolve zero se a area da tabela publicar
`Nenhum registro encontrado.` Uma tabela paginada nao suportada
e recusada com `paginacao_dos_retornos_nao_observada`, nunca
publicada como lista completa. Nada e inferido sobre dados nao
presentes.

## Saida e orcamento

JSON: `unidade`, `mes` no formato `AAAA-MM`,
`origem: "web"`, `total_declarado`, `faixa_observada`
(`null` se vazio, senao `inicio` e `fim`), `retornos`
(array) e `requisicoes`. Cada retorno publica `situacao`,
`numero`, `unidade`, `data_programada`, `data_retorno`
e `prazo_restante`; os campos nao observados sao `null`.

Sem relogin, a consulta do mes mostrado custa **3 GET**:
Controle, menu e Ver todo o mes. Cada passo pelo calendario acrescenta
1 GET; nao ha retry. Sem `--mes`, uma tela deslocada pode
exigir os mesmos GETs de navegacao ate o mes em que o SEI marca
`diaHoje`; se o marcador nao for encontrado, recusa sem o GET mensal.
Recusas de link ou calendario deixam de pedir a proxima tela. O relogin opcional das leituras acrescenta suas
requisicoes observadas no total; nao e segunda tentativa do GET de
retornos contra uma mesma resposta de erro.

Erros tipados: `mes_dos_retornos_invalido`,
`link_de_retornos_ausente`, `link_de_retornos_ambiguo`,
`link_de_retornos_fora_da_instalacao`,
`calendario_dos_retornos_desconhecido`,
`mes_dos_retornos_inacessivel`,
`tela_de_retornos_desconhecida`,
`lista_de_retornos_incompleta`,
`paginacao_dos_retornos_nao_observada`,
`unidade_diferente_da_solicitada`. Falhas de transporte preservam
a mensagem e o status HTTP, sem exibir cookie nem URL assinada.

## Evidencia e limites

Observado no [Todos @323eb01](https://github.com/franklinbaldo/todos/blob/323eb01/src/todos/retorno_programado.py)
e nos [testes](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_unidade_listar_retornos_010.py).
A decisao esta em
[unidade-retornos](../../docs/decisoes/unidade-listar-retornos.md)
e a observacao em
[Retorno Programado](../../docs/como-o-sei-funciona/retorno-programado.md).
O replay das cassettes reais foi **PASS 14/14** em sete instalacoes
([relatorio](../../docs/validacao-replay/2026-11-05-unidade-retornos.md));
isso nao equivale a validacao live de GO/PEN3.

## Falhas tipadas

| Subcódigo | Fato verificável |
| --- | --- |
| `mes_dos_retornos_invalido` | Argumento AAAA-MM ausente de formato válido |
| `link_de_retornos_ausente` | Nenhum link da operação no Controle |
| `link_de_retornos_ambiguo` | Mais de um link da operação no mesmo Controle |
| `link_de_retornos_fora_da_instalacao` | Link de retorno não passa nas regras de origem e assinatura |
| `calendario_dos_retornos_desconhecido` | Calendário não publicou mês ou marcador inequívoco |
| `mes_dos_retornos_inacessivel` | Mês não alcançável em até 24 passos publicados |
| `tela_de_retornos_desconhecida` | Tabelas, cabeçalhos ou mês vazio sem forma reconhecida |
| `lista_de_retornos_incompleta` | Total declarado em legenda não corresponde às linhas |
| `paginacao_dos_retornos_nao_observada` | Paginador independente sem derivação segura |
| `unidade_diferente_da_solicitada` | Controle comprovou unidade diferente de --unidade |
