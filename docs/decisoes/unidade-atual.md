---
type: ImplementationDecision
title: Identificação passiva da unidade ativa pela página de controle
operation: unidade.atual
implementation: partial
status: adopted-for-scope
scope: leitura passiva do rotulo de unidade ativa publicado na pagina Web
selected_strategy: web-control-get
alternatives: web-control-get,web-switch-page,rest-user-units
evidence_level: primeiro-html-da-caixa-capturado
live_validation: nao-realizada
---

# Unidade ativa sem trocar a sessão

## Alternativas consideradas

- **web-control-get:** aproveitar a página autorizada Controle de
  Processos, que já publica `lnkInfraUnidade`.
- **web-switch-page:** abrir a interface de troca para descobrir
  outras unidades disponíveis.
- **rest-user-units:** consultar unidades do módulo REST opcional.

## Comparação

O primeiro caminho exige somente um GET já utilizado pelo IES e
não altera a sessão. A página de troca e o REST responderiam perguntas
diferentes, com custos e permissões próprios. Não se deve apresentar
o rótulo da unidade como identificador interno.

## Decisão

Oferecer `ies unidade atual` apenas para identificar o texto
publicado na tela ativa. Sem POST, troca ou varredura de outros
endpoints. Se o marcador faltar ou a unidade exibida contrariar
a solicitação explícita, falhar com erro específico.

## Evidências verificáveis

- [Contrato](../../contracts/cli/unidade-atual.md)
- [Evidência derivada](../../contracts/evidencias/unidade-atual.md)
- [Testes da CLI](../../tests/cli_contract.rs)
- [Parser da caixa](../../tests/replay_unidade.rs)

## Limitações e revisão

Não implementa `usuario listar-unidades`, nem seleção/troca de
unidade. A unidade ativa observada pode mudar entre requisições;
uma leitura isolada não garante identidade para operações futuras.
Revisar o escopo com evidência real do menu de troca e autorização.
