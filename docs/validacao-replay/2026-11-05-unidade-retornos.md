---
type: SEIObservation
title: Replay de retornos programados nas sete instalacoes de treinamento
description: 14 paginas mensais conferidas com as cassettes originais, SHA-1 e SHA-256, sem rede SEI
---

# Replay do acervo — `unidade retornos` — 05/11/2026

**PASS** — [workflow do replay](https://github.com/franklinbaldo/ies/actions/runs/42096471505).
O teste Rust `unidade_retornos::acervo::replay_retornos_no_acervo`
executou em `scripts/replay` com seis ZIPs originais, cobrindo os sete
cards `unidade-listar-retornos-*`. Foram conferidos SHA-1 dos ZIPs,
SHA-256 de cada membro JSON, `complete`, schema e origem.

| Instalação | Meses completos no acervo | Retornos nas duas leituras | Recusas |
| --- | ---: | --- | ---: |
| UnB | 2 | 2 e 2 | 0 |
| UFRJ | 2 | 0 e 1 | 0 |
| Goiás | 2 | 0 e 1 | 0 |
| PEN1 | 2 | 0 e 1 | 0 |
| PEN2 | 2 | 0 e 1 | 0 |
| PEN3 | 2 | 0 e 2 | 0 |
| PEN4 | 2 | 0 e 1 | 0 |
| **Total** | **14** | **14/14 lidas** | **0** |

**RED observado no replay antes da correção:** [execução anterior](https://github.com/franklinbaldo/ies/actions/runs/42095213665):
7 leituras aceitas e 7 recusadas como `lista_de_retornos_incompleta`.
A inspeção estrutural, sem imprimir URLs nem HTML, identificou que a
listagem completa de um mês pode publicar somente `aguardando` dentro
de `divAguardandoAreaTabela`, sem uma tabela `devolver`. Esta forma
foi observada nas sete instalações. O leitor passou a aceitar a forma
específica, sem aceitar uma tabela isolada sem a moldura mensal.
Os testes sintéticos preservam essa recusa e os orçamentos de GET.

**Limite:** o replay não envia requisições à instalação (0 GET/POST
ao vivo), não demonstra login, sessão atual, paginação independente
das duas tabelas ou procura de `diaHoje` em mês vizinho. Isso requer
evidência adicional. Nenhuma escrita foi realizada no SEI.

## Navegacao do calendario: duplicatas mensais reais

Uma segunda leitura do **mesmo acervo original**,
[run 42120788751](https://github.com/franklinbaldo/ies/actions/runs/42120788751),
conferiu os GETs de calendario sem imprimir URLs, cookies nem hashes.
Nas sete instalacoes, a maioria das telas publicou **duas ancoras
mensais e uma URL mensal distinta**. O parser antigo recusava a segunda
ancora, mesmo que o `href` fosse literalmente igual.

Foi adicionado teste sintetico reproduzindo as duas ancoras com href
identico e orcamento de 3 GET. **RED: 1 falha** (1546 passando)
no [CI 42121807194](https://github.com/franklinbaldo/ies/actions/runs/42121807194),
antes de corrigir a selecao mensal. O teste complementar comprova
que duas URLs assinadas diferentes continuam recusadas e que a
recusa nao dispara consulta de listagem extra. O
[CI com a correcao](https://github.com/franklinbaldo/ies/actions/runs/42122485266)
aprovou a suite Rust e demais gates.

## Contrato de navegacao multissalto (teste sintetico)

A regressao complementar em
`src/unidade_retornos_web/tests/saltos.rs` cobre calendario
dois meses atras e dois meses adiante do `diaHoje` servido pelo SEI,
exigindo, a cada passo, URL assinada ja publicada na pagina. O custo
afirmado e de **5 GETs** (Controle + calendario inicial + 2 transicoes
+ consulta mensal), **zero POST**, sem restaurar a tela. Esses dois
testes exercitam o codigo de navegacao com respostas sinteticas;
**nao sao replay de uma sequencia real de calendario**. O replay
obrigatorio do CI agora confere 14 HTMLs mensais **e 49 respostas
de calendario** nas sete instalacoes, pelo extrator de producao.
A leitura ao vivo **sem `--mes`** em GO e PEN3 esta registrada no
[relatorio live](../validacao-live/2026-11-05-unidade-retornos.md).

## Filtro de acao do calendario — regressao resolvida

No [CI 42150016425](https://github.com/franklinbaldo/ies/actions/runs/42150016425),
o replay ampliado recusou o calendario UnB no passo 17 como
`link_de_retornos_fora_da_instalacao`. O diagnostico **agregado,
sem URLs nem hashes**, identificou um link de outra acao entre
39 ancoras que mencionavam `retorno_programado_listar`.
A mera ocorrencia dessa palavra num `href` nao identifica a acao
da ancora. A selecao passou a exigir
`acao=retorno_programado_listar`; depois disso, a validacao
estrita de origem, parametros e assinatura continua obrigatoria.

O teste sintetico `link_de_outra_acao_com_origem_retornos_nao_bloqueia_calendario`
foi publicado antes da correcao: **RED comprovado**, com
**1 teste falhando, 1.648 passando e 18 ignorados** no
[CI 42151963795](https://github.com/franklinbaldo/ies/actions/runs/42151963795).
Ele exige leitura completa em **3 GET contando Controle**, ignorando
o link alheio sem executar JavaScript, POST ou retry.

**GREEN da correcao:** [CI 42152204051](https://github.com/franklinbaldo/ies/actions/runs/42152204051),
5/5 jobs PASS. Replay obrigatorio: **49 calendarios** e **14
paginas mensais lidas, zero recusas**, com ZIPs e membros conferidos.
Este ensaio foi inteiramente offline: **zero requisicoes reais ao SEI
e zero escrita ou residuo**. Nao comprova, por si, o caso live de
calendario inicialmente deslocado, que continua NOT_RUN.
