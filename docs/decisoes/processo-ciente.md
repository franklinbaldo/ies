---
type: ImplementationDecision
title: Ciência no processo com GET único e prova na lista
operation: processo.ciente
implementation: partial
status: adopted-for-scope
scope: abertura Web, prévia, GET único e reconciliação por atividade na Lista de Ciências
selected_strategy: arvore-link-servido-funcao-servida-unico-get-lista-relida
alternatives: arvore-link-servido-funcao-servida-unico-get-lista-relida,assumir-sucesso-pelo-html-do-aceite,usar-historico-generico,rest-wssei
evidence_level: documentacao-todos-e-testes-sinteticos-red-green
live_validation: not-run
---

# Ciência pela Lista de Ciências do SEI — #271

## Contexto

O SEI oferece `cienciaProcesso()` na barra da árvore, mas
`procedimento_ciencia` é um **GET mutador**, não uma tela de prévia.
O aceite HTML atualiza a árvore e publica o identificador
`id_atividade` da ciência. Essa resposta não prova o registro:
a lista de ciências relida deve conter o checkbox dessa atividade.
Para documento, há ação diferente (`documento_ciencia`) e marca de
recarga diferente, sem compartilhar uma conclusão indevida.

## Alternativas consideradas

1. **Link observado, GET mutador único e lista relida** (escolhido): só
   publicar a atividade encontrada no aceite e na lista recém-lida.
2. **Assumir sucesso no HTML do aceite**: não comprova o registro.
3. **Consultar apenas o histórico genérico**: não prova a ciência.
4. **REST/WSSEI**: sem evidência que legitime a escolha.

## Comparação

| Critério | Lista relida | Aceite apenas | Histórico | REST |
| --- | --- | --- | --- | --- |
| Escrita sem retry | sim | sim | sim | não observado |
| Ciência provada pela atividade | sim | não | não | não observado |
| URL derivada do SEI | sim | sim | sim | não observado |

## Decisão

Conferir primeiro a identidade NUP ↔ id e unidade na abertura Web,
e o botão servido na barra. Sem `--confirmar`, não abrir o GET de
ciência. Com confirmação, abrir a tela do nó, ler a URL da própria
função sem executar JavaScript, e fazer uma única requisição mutadora.
Depois da tentativa, mesmo em timeout ou 302, reabrir a árvore uma vez.
Só publicar sucesso se a nova lista da própria árvore contiver
a atividade exata do aceite. Quando o aceite não identifica atividade
ou a verificação falha, sair com **3**, não repetir o GET mutador.

O leitor das tags utiliza `html_atributos`; o dos literais não usa
regex nem executa JS. Reaproveitar a abertura, o contador de
requisições e o transporte de `processo concluir`. Os endereços
permanecem redigidos, dentro da origem autorizada. Toda resposta
externa exige conferência da identidade, não inferência da URL.

## Alternativas rejeitadas

Aceitar o HTML 200 do GET como sucesso: o próprio SEI pode recusá-lo
com validação. Ler andamento genérico: não prova o registro do usuário
na atividade informada. Tentar novamente após timeout: pode duplicar
a escrita. Montar URL: perde a assinatura da sessão. REST/WSSEI:
sem observação que fundamentasse sua seleção.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-ciente.md) e
  [matriz de evidências](../../contracts/evidencias/processo-ciente.md).
- [Teste Rust de superfície](../../tests/cli_processo_ciente.rs)
  e [relatório live](../validacao-live/2026-11-05-processo-ciente.md).

## Limitações e revisão

## Limitações

O texto da função foi visto na documentação empírica do Todos e a
simulação local não substitui a leitura das cassettes completas. A
segunda ciência do dia deve ser recusada pelo SEI, mas o seu erro
tipado e o motivo ainda exigem captura e testes. A verificação
do usuário da sessão não pode ser afirmada só pelo valor do checkbox;
o fluxo comprova somente a presença da atividade vinculada ao processo.
A CLI de documento ainda não está integrada. Testes de rede sintéticos,
replay e validação live permanecem pendentes.

## Causa observada do GET mutador (06/11/2026, #271)

A ciência por processo também emprega **um GET de escrita**, como a ciência
por documento. Antes desta correção, `resposta.ok()` descartava a causa
original (HTTP 403, validação Infra com HTTP 200 ou falha de rede),
impedindo o diagnóstico quando a releitura não encontrava a atividade.

A decisão é preservar `FalhaDoAceiteProcesso` dentro do
`EfeitoIncerto`, sem retry e sem converter status HTTP em confirmação de
sucesso; somente a lista relida pode provar a atividade. O RED
[42215714854](https://github.com/franklinbaldo/ies/actions/runs/42215714854)
mostrou três falhas antes da implementação, com 1.711 PASS. O GREEN
depende do CI do novo head. Ensaio em treinamento e replay específico
continuam NOT_RUN, com zero requisições SEI e zero resíduos.

## Regressão: aceite JS inerte não comprova ciência (06/11/2026)

A atividade do aceite é reconhecida somente numa chamada estática
`atualizarArvore` em código de script, fora de comentários e literais.
Um `<script>` aninhado em `<template>` não é prova de execução. A
extração reaproveita o lexer conservador já usado para ciência de
documento; sem `eval`, execução de JavaScript ou expressão regular.
Se não há atividade comprovável, o GET mutador **não é repetido** e
a releitura mantém o resultado coma escrita não confirmada (saída 1).

O RED anterior ao código foi comprovado no
[CI 42218975613](https://github.com/franklinbaldo/ies/actions/runs/42218975613):
**3 falhas novas, 1.714 testes aprovados e 22 ignorados preexistentes**.
Comentários, literais de string e scripts de template foram os três
contraexemplos que antes devolviam a atividade fictícia. O GREEN deve
ser conferido no CI do head atualizado da PR #317. Replay específico
e validação em treinamento permanecem NOT_RUN.


## Aceite em ramo condicional — correção após revisão 🦅 (06/11/2026)

A profundidade de chaves do leitor estático não prova execução: tanto
`function inicializar(){ atualizarArvore(...) }` quanto
`if(false){ atualizarArvore(...) }` têm profundidade 1. A chamada
também pode estar em função arbitrária não invocada, condição sem chaves
ou expressão cujo efeito não foi comprovado. Portanto, admitir qualquer
posição lexical fora de comentários e literais era inseguro.

A extração agora admite somente uma instrução independente no nível
principal do script ou no corpo direto da função `inicializar()`
observada na resposta do SEI. Não avalia condições nem executa JS;
quando o controle de fluxo não é comprovado, devolve
`AceiteSemAtividade`. Mesmo que a releitura já contenha o id da
atividade, não o usa para confirmar a escrita: **escrita não confirmada e
saída 1, sem repetir o GET mutador**.

Regressões sintéticas em `src/ciencia/tests.rs` e
`src/ciencia/tests_fluxo.rs` cobrem `if(false)` com e sem chaves,
função não invocada e resposta com lista preexistente. O RED anterior
à correção está no [CI 42223177117](https://github.com/franklinbaldo/ies/actions/runs/42223177117):
**3 FAIL / 1.717 PASS / 22 ignored preexistentes**. GREEN confirmado
no [CI 42224041949](https://github.com/franklinbaldo/ies/actions/runs/42224041949)
do head `e646671`, com **5/5 jobs PASS** (prek, release e três
replays gerais). Replay específico de ciência continua NOT_RUN.


## Instrução inalcançável após retorno — regressão #271 (06/11/2026)

O reconhecimento por nível lexical e assinatura `inicializar()` ainda
aceitava `return; atualizarArvore(...)` dentro da função. O ponto e
vírgula fazia a chamada seguinte parecer independente, embora o retorno
impedisse sua execução; um ID preexistente na lista podia produzir
confirmação falsa. A decisão restringe a prova à **primeira instrução
executável** do próprio script (chamada direta) ou do corpo direto
da função `inicializar()`. Comentários e espaços são tolerados pelo
leitor lexical, mas qualquer outra instrução anterior faz recusar o
aceite, sem executar JavaScript ou inferir o fluxo de controle. Se uma
instalação servir forma diferente, o fluxo falha fechado (saída 1),
até observação específica e novo contrato.

**RED observado antes do código:** [CI 42231850140](https://github.com/franklinbaldo/ies/actions/runs/42231850140),
**2 FAIL / 1.724 PASS / 22 ignored preexistentes**. A regressão unitária
detectou falso `Ok("42457")` após `return;` e a de fluxo detectou
`confirmado=true` quando a lista continha ciência anterior.
O teste de fluxo exige **6 GET, exatamente 1 mutador e nenhuma consulta
à lista sem atividade do aceite comprovada**. GREEN do head posterior
é verificado separadamente no CI; replay específico e live NOT_RUN.


## Script de dados ou externo não prova aceite (#271, 06/11/2026)

O parser da resposta de ciência percorria todo `<script>`, inclusive
nós com `type=application/json` e com `src` externo. Embora o navegador
não execute o corpo inline desses scripts, a leitura estática extraía
o `id_atividade` e poderia associar uma ciência antiga ao GET atual.
A peça `html_atributos` passa a distinguir script inline executável de
conteúdo inerte, antes de aceitar qualquer chamada `atualizarArvore`.
A guarda segue conservadora, sem executar JS nem ampliar a rede.
O teste RED no [CI 42268070022](https://github.com/franklinbaldo/ies/actions/runs/42268070022)
registrou **1 FAIL / 1.739 PASS / 22 ignored preexistentes**:
`application/json` retornava falsamente `Ok("42457")`.
O teste de fluxo exige uma releitura sem consultar Lista de Ciências
quando o aceite não publicou atividade válida (**6 GET, 1 mutador,
0 retry, saída 1**). GREEN e live dependem de validação posterior.


## Argumento avaliado antes da recarga — regressão #271 (06/11/2026)

O aceite medido no oráculo Todos (test_ciclo_ciencia_010.py,
test_documento_ciente_010.py @323eb01) chama atualizarArvore('url')
com **um único argumento literal**. O leitor permitia também
vírgula depois do literal. Em JavaScript, os argumentos são
avaliados antes de a função ser chamada: um segundo argumento
que lança exceção impede a recarga, embora o leitor retornasse
falsamente a atividade.

**RED pré-implementação:** [CI 42278019363](https://github.com/franklinbaldo/ies/actions/runs/42278019363),
**2 FAIL / 1.766 PASS / 22 ignored preexistentes**. O teste unitário
observou Ok("42457") em vez de AceiteSemAtividade; o teste de
fluxo tentou consultar a Lista de Ciências antiga além das seis
requisições do roteiro.

A correção restringe o literal à forma de um único argumento
seguido de parêntese de fechamento; sem executar JavaScript.
Na resposta de argumento adicional, há releitura independente
da árvore, sem consulta de lista por id não comprovado, sem repetir
o GET mutador; o fluxo responde escrita não confirmada (saída 1).
O GREEN precisa ser comprovado no CI do novo head; replay específico
e live em treinamento continuam NOT_RUN.
