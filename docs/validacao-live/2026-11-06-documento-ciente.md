---
type: SEIObservation
title: Validação offline e estado live de ciência documental em 06/11/2026
description: Testes sintéticos e pendências de treinamento da issue 271
---

# Ciência documental — #271, PR #317

| Etapa | Resultado | Requisições ao SEI |
| --- | --- | ---: |
| Observação: Todos @323eb01, UnB 4.0.9 | REFERÊNCIA | 0 |
| RED da superfície CLI, GitHub Actions [42193308344](https://github.com/franklinbaldo/ies/actions/runs/42193308344) | FAIL esperado: 3 falhas novas, 0 pass, 0 ignored | 0 |
| Suíte Rust, [CI 42197217607](https://github.com/franklinbaldo/ies/actions/runs/42197217607) | PASS nos testes da ciência; Clippy detectou documentação e estilo, corrigidos em commits posteriores | 0 |
| RED de segurança: JS inerte (primeira rodada), [CI 42200846375](https://github.com/franklinbaldo/ies/actions/runs/42200846375) | FAIL esperado: 2 falhas, 1.690 PASS, 22 ignores preexistentes | 0 |
| RED com quebras de linha reais, [CI 42201110316](https://github.com/franklinbaldo/ies/actions/runs/42201110316) | CANCELLED por push subsequente; não atribuir contagem | 0 |
| RED do bloqueante 403/Infra/template aninhado, [CI 42208759616](https://github.com/franklinbaldo/ies/actions/runs/42208759616) | CANCELLED após push de implementação, **sem contagem RED confirmada** | 0 |
| GREEN da correção do aceite | Consultar o CI do head atual da [PR #317](https://github.com/franklinbaldo/ies/pull/317); não presumir resultado | 0 |
| Replay das cassettes de ciência | NOT_RUN | 0 |
| Prévia na UnB treinamento | NOT_RUN | 0 |
| Prévia em segunda instalação de treinamento | NOT_RUN | 0 |
| Um único `--confirmar` (treinamento) | NOT_RUN | 0 |
| Verificação por releitura após escrita live | NOT_RUN | 0 |

Nenhum acesso ao SEI ou escrita ao vivo ocorreu nesta fatia;
**zero resíduos novos**. O orçamento dos testes sintéticos é 3 GET
na prévia, 7 GET em confirmação e prova, 6 GET quando o aceite não
fornece atividade; **apenas um** dos GETs pode mutar o documento.

Não há comprovação da segunda ciência e do replay completo; permanecem
pendências da issue. Nunca executar em produção. A sequência segura de
treinamento é prévia ao vivo → uma confirmação → leitura de prova,
com registro do documento e do resíduo, caso seja possível completá-la.

O teste de regressão `template_inerte_nao_autoriza_get_mutador` comprova que
texto inerte em `<template>` não autoriza GET de escrita. O leitor agora
considera somente scripts inline ativos e ignora scripts de dados e fontes
externas sem prova observada. A tentativa de RED isolado desse novo teste
foi cancelada no [CI 42197133423](https://github.com/franklinbaldo/ies/actions/runs/42197133423)
antes de obter resultado; **RED novo: NOT_RUN**, não se confunde com o
RED inicial da superfície da CLI (três falhas já documentadas).

A revisão do Gavião identificou GET mutador indevido quando a função de
ciência só aparecia num comentário de JavaScript ou num ramo inativo.
O RED registrou duas falhas reais. A correção só aceita duas formas
estáticas observadas: variável no topo do script e função 4.0 com
atribuição de destino diretamente no corpo. Comentários, literais,
templates, declarações sob condição e trechos sem forma reconhecida
recusam antes da escrita. Novos testes exigem **4 GET de preparação,
zero GET mutadores** nesses cenários; não se executa JavaScript.

## Retomada 🦊 — causa do GET mutador

A revisão do Gavião no head `97a8e35` encontrou perda do erro de
`rede.get_sem_seguir` causada por `resposta.ok()`. Agora o erro de
aceite é tipado e anexado ao efeito incerto, distinguindo transporte,
recusa HTTP, validação Infra e atividade não reconhecida, sem repetir o
GET. A saída JSON e a linha humana preservam status e mensagem saneada.
Testes sintéticos afirmam 403 (6 GET; 1 mutador), Infra 200 (6 GET;
1 mutador), falha de rede (6 GET; 1 mutador) e recusa de
`<template><script>` (4 GET; 0 mutador). O run RED desta rodada foi
cancelado sem evidência de contagem; **RED desta rodada: NOT_RUN**.

**Live:** prévia NOT_RUN (0), confirmação NOT_RUN (0), releitura NOT_RUN (0),
replay específico NOT_RUN. Nenhuma requisição ao SEI ou resíduo criado;
a escrita ao vivo continua dependente de prévia autenticada e confirmação
única em treinamento, seguida de prova por leitura.


## Regressão da Lista de Ciências sob template ancestral (06/11/2026)

Observação estrutural: `src/ciencia.rs::atividade_na_lista` já recusava
checkbox sob `<template>` **dentro** da tabela, mas não o caso inverso:
a tabela `tblAndamentos` inteira sob `<template>`. O navegador não
exibe o fragmento inerte, embora o parser o encontre pelo ID.
**RED comprovado** na suíte Rust do [run 42286359093](https://github.com/franklinbaldo/ies/actions/runs/42286359093):
**1 teste falhou, 1.768 passaram, 22 ignorados preexistentes**.
O hook Rustfmt também apontou apenas estilo no teste novo, corrigido no GREEN.
A correção recusa ancestral `<template>` da tabela e o teste de fluxo
exige saída 3 sem sucesso falso, com 7 GET totais, **1 GET mutador**
e **0 retry** (orçamento sintético). Não há mudança na prévia nem no
protocolo Web. GREEN: consultar o CI do head publicado desta PR.

**Live:** UnB NOT_RUN (0 pedidos), segunda instalação NOT_RUN (0),
`--confirmar` NOT_RUN (0 GET mutadores), releitura NOT_RUN (0);
replay dedicado de ciência NOT_RUN. **0 escritas e 0 resíduos novos**.

## Continuação da Raposa — PR #337 (06/11/2026)

A PR #317 foi integrada na `main` como `dcec619`; esta fatia usa
nova branch baseada na `main`, mantendo a issue #271 aberta.

**Observação:** no Todos, segunda ciência no processo é recusada se já
houve ciência **no dia**, ao passo que no documento a recusa é da
segunda ciência **do documento**, sem reinício diário
(`tests/test_ciclo_ciencia_010.py` e `tests/test_documento_ciente_010.py`
@323eb01). Os aceites recusados trazem `txaInfraValidacao` HTTP 200.
Esse conteúdo é evidência do Todos, não de execução live desta fatia.

**RED comprovado em CI** [42294900770](https://github.com/franklinbaldo/ies/actions/runs/42294900770):
2 FAIL novos / 1.770 PASS / 22 ignored preexistentes, antes da correção.
Um checkbox dentro de `<noscript>` e uma tabela inteira sob
`<noscript>` eram tratados como prova; ambos são inertes com scripts
habilitados. A recusa agora preserva o limite de uma escrita e não
acrescenta GET.

**Testes adicionais:** ciência repetida com mensagem Infra 200 tipada;
documento com tabela inerte e atividade idêntica ao aceite. Orçamento
sintético: 6 GET / 1 mutador / 0 retry para aceite recusado, ou 7 GET /
1 mutador / 0 retry para lista sem prova. A saída é 3 (efeito incerto),
sem alegar ciência nova e sem consultar lista antiga por um aceite
não comprovado.

**Live nesta continuação:** prévia UnB NOT_RUN (0), prévia segunda
instalação NOT_RUN (0), confirmação única NOT_RUN (0 mutadores),
verificação live NOT_RUN (0). Replay específico de `processo-ciente`/
`documento-ciente` NOT_RUN. **0 requisições reais, 0 escritas e 0
resíduos novos no SEI.** Os replays genéricos do CI não suprem essa prova.
A issue permanece WIP até replay dedicado e ensaio seguro em treinamento.

## 🦊 Continuação — script do aceite em `noscript` (#271)

O parser do IES expunha a chamada `atualizarArvore` inserida dentro de
`<noscript><script>...`, apesar de o navegador com JS habilitado não
executá-la. O oráculo estrutural é a exigência de script ativo no aceite:
a URL de uma ciência antiga não pode comprovar o GET atual.

**RED comprovado:** [CI 42302428489](https://github.com/franklinbaldo/ies/actions/runs/42302428489):
`script_dentro_de_noscript_nao_prova_aceite` devolvia indevidamente
`Ok("42457")`; `script_inerte_em_noscript_nao_confirma_ciencia_antiga`
tentava GET de lista além do roteiro sintético.
**2 FAIL novos / 1.775 PASS / 22 ignorados anteriores**. Não houve
`#[ignore]` novo para mascarar a falha. Correção: recusa estática de
script em `noscript`, releitura sem lista antiga e saída 3; orçamento
sintético 6 GET, um único mutador, zero retry.

Foi acrescentado replay offline específico com 14 cards das sete
instalações e hashes verificados pelo leitor comum do acervo. **Resultado
efetivo do replay: pendente da execução no runner**; não marcar PASS
antes dos logs. Prévia live UnB **NOT_RUN (0 req)**; segunda instalação
**NOT_RUN (0)**; confirmação **NOT_RUN (0 mutadores)**; releitura live
**NOT_RUN (0)**. Até aqui, **0 requisições SEI, 0 escritas, 0 resíduos**.

## Resultado real do replay específico — 06/11/2026

[GitHub Actions 42303368527](https://github.com/franklinbaldo/ies/actions/runs/42303368527)
**PASS técnico**, execução offline de 14 cassettes verificadas, 42
respostas classificadas, sem acesso ao SEI. **Não foi PASS funcional:**
28 respostas relacionadas ao aceite retornaram
`aceite_da_ciencia_sem_atividade` e 14 respostas de Lista de Ciências
ficaram não aplicáveis (`aceite_sem_atividade_observada`).
**Zero atividades aceitas.** Ver
[relatório detalhado](../validacao-replay/2026-11-06-ciencia.md).
Investigar formato do HTML gravado e sequência das respostas antes
de alegar conformidade, sem relaxar segurança ou inventar prova.

Prévia UnB **NOT_RUN (0 req)**; segunda instalação **NOT_RUN (0)**;
um `--confirmar` autorizado **NOT_RUN (0 mutadores)**; releitura live
**NOT_RUN (0)**; **0 requisições reais, 0 escritas e 0 resíduos**.
