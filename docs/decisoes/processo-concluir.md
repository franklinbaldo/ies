---
type: ImplementationDecision
title: Conclusão do processo pela ação publicada na barra, verificada pela situação da árvore
operation: processo.concluir
implementation: partial
status: adopted-for-scope
scope: ies processo concluir NUP com prévia sem requisição além da abertura; --confirmar com uma escrita (POST da tela Conclusão de Processo no SEI 4.1 e 5.0, GET da função concluirProcesso no 4.0) e verificação pela situação aberto_em de uma leitura nova da árvore
selected_strategy: web-acao-da-barra-verificada-pela-situacao
alternatives: web-acao-da-barra-verificada-pela-situacao,web-verificada-pela-barra,web-so-forma-tela,rest-wssei
evidence_level: cassettes-publicas-sete-instalacoes-e-testes-sinteticos
live_validation: escrita-pass-ufrj-0415-tela-unb-0409-funcao
---

# Concluir o processo (Refs #172)

## Contexto

Primeira escrita de estado do processo. A barra do nó `PROCESSO` publica a
ação em duas formas (observação em
[conclusão do processo](../como-o-sei-funciona/conclusao-do-processo.md)):
link para a tela "Conclusão de Processo" (4.1 e 5.0, cinco instalações) ou
`concluirProcesso()` com a URL no corpo da função na tela do nó (4.0, UnB e
PEN1 antes da 5.0.4). Nas 219 conclusões gravadas, a situação `aberto_em`
da árvore seguinte perdeu exatamente a unidade da sessão.

## Alternativas consideradas

1. **Ação da barra, verificada pela situação** (escolhida): as duas formas,
   cada uma pelo que a tela publica; sucesso só se `aberto_em` relido não
   lista a unidade da sessão.
2. **Verificada pela barra**, como o Todos: concluído oferece
   `reabrirProcesso()` e não oferece concluir. Lê a mesma árvore, mas a
   barra diz o que o usuário pode fazer, não onde o processo está aberto;
   com o processo aberto noutra unidade a leitura fica indireta.
3. **Só a forma da tela**: menos código, mas deixa a UnB 4.0.9 (critério de
   aceite da issue) sem conclusão.
4. **REST/WSSEI**: fora da prioridade ([ADR 0001](../arquitetura/0001-rotas-por-evidencia.md)).

## Comparação

| Critério | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| Instalações cobertas pela evidência | 7 | 7 | 5 | 0 |
| Verificação pelo fato pedido (unidade fechada) | sim | indireta | sim | — |
| Requisições a frio (`tela`/`funcao`) | 5+1 POST / 6 | iguais | 5+1 POST / — | — |
| URL montada pelo IES | nenhuma | nenhuma | nenhuma | — |

## Decisão

- A abertura é a de `ies processo NUP` sem extras e **sem o passo 1**
  (link guardado da árvore), porque ele não publica a unidade; a conclusão
  vale para a unidade de `lnkInfraUnidade`. A verificação é uma leitura
  normal, que em geral usa esse link (1 GET).
- Prévia: só a abertura. A tela de conclusão não é pedida (o efeito de
  abri-la sem salvar não foi medido) nem a tela do nó (na forma `funcao` a
  URL de lá já é a escrita).
- `aberto_em` não lido antes da escrita é recusa: o que não se pode
  verificar não se escreve. Sem a unidade da sessão e sem a ação na barra é
  sucesso sem escrita (`alterado: false`).
- Forma `tela`: formulário conferido por lista fechada de controles, `S`
  marcado de saída, nada de reabertura preenchido; POST pelo primitivo com
  a `const` da operação. Forma `funcao`: só a forma de três linhas
  observada; o GET da URL é a escrita. Uma escrita, nunca repetida; a
  reconciliação segue a tabela do [contrato](../../contracts/cli/processo-concluir.md).
- `EfeitoAvaliado::AlteraEstadoDoProcesso` é caso novo do primitivo:
  não é preferência do usuário nem estado de sessão.
- Reabrir fica fora: observado nas sete instalações como
  `reabrirProcesso()`, mas é outro comando da CLI pública e precisa de
  proposta ao proprietário (plano no relatório live).

Descartado: seguir o 302 do POST (o cliente não segue redirect); ler a
situação na resposta da forma `funcao` em vez de uma leitura nova (não se
sabe se a página reflete o gravado); aceitar "Concluir e agendar".

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-concluir.md) e
  [evidência da operação](../../contracts/evidencias/processo-concluir.md)
- [Observação estrutural](../como-o-sei-funciona/conclusao-do-processo.md)
- [Superfície da CLI](../../tests/cli_processo_concluir.rs)
- [Fluxo, sequências e reconciliação](../../src/processo_concluir/tests_fluxo.rs),
  [tela e resposta](../../src/processo_concluir/tests_tela.rs),
  [barra e função](../../src/arvore_processo/tests_concluir.rs)
- [Replay do acervo](../../src/processo_concluir/acervo.rs) e
  [relatório live com o plano de escrita](../validacao-live/2026-11-04-processo-concluir.md)

## Limitações e revisão

Sem corpo de POST nas cassettes: os campos vêm do formulário servido e do
teste do Todos. A forma da PEN1 5.0.4 não foi observada. A resposta da
forma `funcao` não tem sinal próprio (só HTML 200); a verificação decide.
Escrita live **PASS** na UFRJ 4.1.5 (`tela`) e na UnB 4.0.9 (`funcao`),
autorizada pelo proprietário ([relatório](../validacao-live/2026-11-04-processo-concluir.md)).
Revisar se o ensaio mostrar: tela de
conclusão com outro controle, resposta diferente de 302 na forma `tela`,
situação que muda só depois de outra navegação, siglas de `lnkInfraUnidade`
diferentes das de `aberto_em`, ou recusa do SEI na própria tela.
