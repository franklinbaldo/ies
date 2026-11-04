---
type: ImplementationDecision
title: Reabertura do processo pela URL que a tela do nó publica, verificada pela situação da árvore
operation: processo.reabrir
implementation: partial
status: adopted-for-scope
scope: ies processo reabrir NUP com prévia sem requisição além da abertura; --confirmar com o GET da tela do nó, o GET da URL procedimento_reabrir lida nela (corpo de reabrirProcesso no SEI 4.0, variável linkReabrirProcesso no 4.1 e 5.0) e verificação pela situação aberto_em de uma leitura nova da árvore
selected_strategy: web-url-da-tela-do-no-verificada-pela-situacao
alternatives: web-url-da-tela-do-no-verificada-pela-situacao,web-so-forma-funcao,web-url-guardada-da-conclusao,rest-wssei
evidence_level: cassettes-publicas-sete-instalacoes-e-testes-sinteticos
live_validation: escrita-pass-ufrj-0415-variavel-unb-0409-funcao
---

# Reabrir o processo (Refs #172)

## Contexto

Pedida pelo proprietário como espelho de
[`processo concluir`](processo-concluir.md), para desfazer a conclusão do
ensaio live e para o fluxo de trabalho comum (concluir por engano). A
barra é a mesma nas sete instalações (`reabrirProcesso()`); a URL está na
tela do nó em duas formas: o corpo da função (4.0) ou a variável
`linkReabrirProcesso` (4.1 e 5.0), observação em
[reabertura do processo](../como-o-sei-funciona/reabertura-do-processo.md).
A primeira leitura do #172 dizia "corpo da função" para as sete; a
releitura das 431 telas do nó gravadas desmentiu isso para as cinco
instalações 4.1/5.0.

## Alternativas consideradas

1. **URL da tela do nó, verificada pela situação** (escolhida): o GET da
   tela do nó (estado atual), a URL numa das duas formas exatas, um GET
   que é a escrita; sucesso só se `aberto_em` relido lista a unidade da
   sessão.
2. **Só a forma da função**: o leitor de `concluirProcesso()` já existia,
   mas deixaria as cinco instalações 4.1/5.0 sem reabrir (lá não há
   função na página).
3. **URL guardada da conclusão**: na forma `funcao` a página do GET de
   concluir também define `reabrirProcesso()`; guardar a URL pouparia 1
   GET, mas só valeria logo depois de um `concluir` do IES e não na forma
   `tela` (o 302 não traz página).
4. **REST/WSSEI**: fora da prioridade ([ADR 0001](../arquitetura/0001-rotas-por-evidencia.md)).

## Comparação

| Critério | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| Instalações cobertas pela evidência | 7 | 2 | 2 (só após concluir) | 0 |
| URL do estado atual | sim | sim | não (de outra página) | — |
| Requisições a frio | 6 GET | 6 GET | 5 GET | — |
| URL montada pelo IES | nenhuma | nenhuma | nenhuma | — |

## Decisão

- Abertura, decisão pela unidade da sessão em `aberto_em` e verificação
  são as de `processo concluir`, pelas mesmas peças
  (`processo_concluir::escrita`: contagem, cookie, tela antes da escrita,
  GET que é a escrita, leitura de verificação); a abertura lê as duas
  ações da barra numa passada (`AcoesDeEstado`).
- O leitor de três linhas de `concluirProcesso()` foi generalizado
  (`urls_das_funcoes`) e serve a `reabrirProcesso()`; a variável é uma
  linha exata `var linkReabrirProcesso = '…';`. As duas formas juntas,
  repetidas ou fora da forma são recusa antes da escrita.
- Prévia: só a abertura; a tela do nó não é pedida (`forma: null` até
  pedi-la). Já aberto na unidade e sem reabrir na barra é sucesso sem
  escrita (`reabertura_oferecida: false`); reabrir oferecido com a unidade
  já em `aberto_em`, ou fora dela sem reabrir, é recusa (combinações não
  gravadas).
- Resposta esperada: HTML 200 sem login nem validação da Infra; um 302 é
  `redirecionamento_na_reabertura`. Uma escrita, nunca repetida; a
  reconciliação segue a tabela do [contrato](../../contracts/cli/processo-reabrir.md).
- A atribuição ao usuário, observada, não é verificada nem publicada: a
  verificação é a unidade, como em concluir.

Descartado: ler a situação na resposta do GET (não se sabe se a página
reflete o gravado); procurar `procedimento_reabrir` em qualquer lugar da
página (só as duas formas exatas).

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-reabrir.md) e
  [evidência da operação](../../contracts/evidencias/processo-reabrir.md)
- [Observação estrutural](../como-o-sei-funciona/reabertura-do-processo.md)
- [Superfície da CLI](../../tests/cli_processo_reabrir.rs)
- [Fluxo, sequências e reconciliação](../../src/processo_reabrir/tests_fluxo.rs),
  [barra e tela do nó](../../src/arvore_processo/tests_reabrir.rs)
- [Replay do acervo](../validacao-replay/2026-11-04-processo-reabrir-acervo.md)
  e [relatório live](../validacao-live/2026-11-04-processo-concluir.md)

## Limitações e revisão

O `.js` que define `reabrirProcesso()` no 4.1/5.0 não foi gravado: que ele
navega para `linkReabrirProcesso` é inferido do efeito. A reabertura com o
processo aberto noutra unidade e a recusa do SEI não foram gravadas.
Escrita live **PASS** na UFRJ 4.1.5 (`variavel`) e na UnB 4.0.9
(`funcao`), com as requisições do orçamento.
Revisar se o ensaio mostrar: resposta diferente de HTML 200, a variável
noutra forma, a função com `confirm` ativo, a PEN1 5.0.4 com outra forma,
ou situação que muda só depois de outra navegação.
