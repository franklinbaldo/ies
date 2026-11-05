---
type: ImplementationDecision
title: Processo marcar por id e nome com leitura completa e escrita fechada até formulário observado
operation: processo.marcar
implementation: partial
status: adopted-for-scope
scope: interpretacao da identidade id#nome e previa de retirada, com atribuicao idempotente e retirada idempotente; sem POST remoto
selected_strategy: reaproveitar-leitor-web-e-falhar-fechado-sem-formulario
alternatives: reaproveitar-leitor-web-e-falhar-fechado-sem-formulario,post-inferido-da-tela,usar-api-rest
evidence_level: estrutura-observada-unb-e-cassettes-de-escrita-nao-reproduzidas
live_validation: not-run-sandbox-sem-acesso-sei
---

# Decisão: `processo marcar` (Refs #255)

## Contexto

A [tela observada na UnB](../como-o-sei-funciona/marcadores-do-processo.md)
e o relatório de [leitura live](../validacao-live/2026-11-04-processo-obter-marcadores.md)
demonstram que o mesmo `id_marcador` aparece em processos distintos;
o rótulo `id#nome` não é identidade da associação ao processo.
O `tblMarcadores` declara seu total, mas pode publicar menos linhas,
portanto ausência só é demonstrada numa lista completa. Os botões de
adição/remoção e a ação servida ainda exigem evidência do POST.

## Alternativas consideradas

1. **Reusar o leitor `processo_obter`**, conferir identidade e unidade,
   oferecer idempotência e prévia e falhar fechado antes da escrita
   (selecionada neste recorte).
2. Inferir um POST a partir do `acaoRemover`, sem reproduzir formulário
   e resultado: rejeitada porque não prova estado posterior nem campos.
3. REST/SOAP: não é a via Web observada nem provou equivalência.

## Comparação

| Critério | Leitor existente | POST inferido | REST |
| --- | --- | --- | --- |
| URL criada artificialmente | não | possivelmente | sim |
| Alteração antes de observar resposta | não | sim | sim |
| `id#nome` e unidade conferidos | sim | incompleto | desconhecido |
| Comportamento demonstrável offline | sim | não | não |
| Prévia sem escrita | sim | incerto | incerto |

## Decisão

Antes de qualquer arquivo, validar número e rótulo. Presença significa
id exato e nome exato; nome divergente é recusa, não substituição.
Ausência só é afirmada se `total_declarado == itens.len()`.
`null` nunca equivale a zero. Retirada de marcador presente retorna
prévia, mas com `--confirmar` falha tipada **antes de qualquer POST**.
Atribuir marcador ausente exige prova do catálogo **da unidade**.
Nenhum novo transporte, cookie próprio ou restaurador de tela.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/processo-marcar.md).
- [Evidência, com limitações explícitas](../../contracts/evidencias/processo-marcar.md).
- [Testes de CLI](../../tests/cli_processo_marcar.rs).
- [Testes sintéticos de decisão](../../src/processo_marcar/tests.rs).
- [Relatório live](../validacao-live/2026-11-05-processo-marcar.md).

## Limitações e revisão

O recorte **não escreve**. A próxima fatia precisa observar formulários
de catálogo, atribuição e retirada da unidade, derivar um envio pelo
`formulario_web`, medir orçamento exato e provar a nova leitura.
Efeito incerto exige saída 3, sem retry. Revisar após replay de
`marcador-escritas-*` e `marcador-ciclo-completo-*` e validação live
em treinamento; nenhuma escrita foi realizada nesta sessão.
