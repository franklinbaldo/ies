---
type: ImplementationDecision
title: Retornos programados da unidade pela tela Web do mes inteiro
operation: unidade.listar_retornos
implementation: partial
status: adopted-for-scope
scope: leitura do mes no Retorno Programado pelo link do menu e links GET de calendario
selected_strategy: web-calendario-mes-inteiro
alternatives: web-calendario-mes-inteiro,web-um-dia-por-vez,rest-opcional
evidence_level: todos-323eb01-tests-sinteticos-replay-7-instalacoes-14-meses-pass
live_validation: fail-go-pen3-2026-11-05
---

# Retornos da unidade: escolher a leitura do mes inteiro (Refs #253)

## Alternativas consideradas

- **Web calendario, mes inteiro (escolhida):** segue o link assinado
  publicado em Controle, navega o calendario somente pelos dias primeiro
  dos meses e abre uma unica vez o `Ver todo o mes`.
- **Um GET para cada dia:** soma ate 31 pedidos, aumenta efeitos e custos
  sem fornecer linhas ou totais mais verificaveis.
- **REST opcional:** nao ha observacao de contrato equivalente nas
  instalacoes de treinamento; contraria Web-first.

## Comparação

O [oraculo Todos 323eb01](https://github.com/franklinbaldo/todos/blob/323eb01/src/todos/retorno_programado.py)
e seus [testes do mes](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_unidade_listar_retornos_010.py)
identificam o menu `retorno_programado_listar`, o `diaHoje`,
as URLs assinadas de `data_atual` e de `data_inicial/data_final`,
e duas tabelas com legendas independentes. A pagina do menu pode
abrir no ultimo dia selecionado, nao no dia de hoje.

Escolhemos 3 GET (Controle + tela + mes) quando ja no mes desejado
e um GET por transicao de mes, com limite 24 passos. A URL e validada
contra a instalacao autorizada antes do cookie; o cliente HTTPS nao
segue redirecionamentos e nao repete. **Nao** fazemos POST para salvar
filtro, nem GET de restauracao.

## Decisão

Cada tabela exige colunas e total da legenda que confira suas linhas.
O primeiro replay conferido encontrou, em todas as sete cassettes,
uma pagina mensal com apenas `aguardando`, dentro de
`divAguardandoAreaTabela` e sem area de `devolver`. Recusar essa
forma real produziria sete falsos `lista_de_retornos_incompleta`.
A excecao so vale na moldura integral da lista, com uma unica
situacao `aguardando` e contagem conferida. Uma tabela isolada,
sem a area observada, continua recusada. Areas explicitamente vazias
continuam valendo como ausencia comprovada.
Mês com texto literal `Nenhum registro encontrado.` na area de tabela
e vazio. A pagina que apresenta paginador nao demonstrado e recusada:
nao afirma total nem completa lista parcial.

`--mes` e validado offline. Sem mes, a implementacao exige a celula
`diaHoje` na tela aberta. Se ela estiver em outro mes ou faltar, a
implementacao atual falha fechado em vez de adivinhar a data da maquina;
a navegacao segura para achar hoje em mes vizinho fica como lacuna.

O relato do RED ocorreu no CI inicial (3 testes falhando, 1361
passando; nao foi possivel compilar no sandbox local sem rustc nem
resolucao DNS). Nao se pode equiparar esse CI a RED local.
O [contrato](../../contracts/cli/unidade-listar-retornos.md), a
[matriz](../matriz-derivacao-cli.md), o
[leitor](../../src/unidade_retornos.rs), o
[fluxo](../../src/unidade_retornos_web.rs) e seus
[testes](../../src/unidade_retornos_web/tests.rs)
delimitam o recorte executavel. O replay offline das sete
cassettes originais passou nas 14 paginas mensais
([relatorio](../validacao-replay/2026-11-05-unidade-retornos.md));
as duas leituras live falharam em treinamento, em GO sem resumo de erro e
na PEN3 com `calendario_dos_retornos_desconhecido` (ver
[relatorio live](../validacao-live/2026-11-05-unidade-retornos.md)). A capacidade
ainda nao esta homologada.

## Evidências verificáveis

- [Contrato normativo](../../contracts/cli/unidade-listar-retornos.md)
- [Matriz de evidência](../../contracts/evidencias/unidade-listar_retornos.md)
- [Teste Rust de CLI](../../tests/unidade_retornos.rs)
- [Teste de navegação e orçamento](../../src/unidade_retornos_web/tests.rs)
- [Replay Rust das sete cassettes](../../src/unidade_retornos/acervo.rs)

## Limitações e revisão

Rever a estrategia ao observar cassettes GO/PEN e telas que paginam as
duas tabelas. Os dois paginadores sao independentes; nao aceitar uma
pagina como prova de cobertura do mes.
