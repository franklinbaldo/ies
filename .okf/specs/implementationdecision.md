---
type: Spec
title: ImplementationDecision
description: Decisão de engenharia que justifica implementação de um contrato de domínio
---

# ImplementationDecision

Uma operação de CLI com `implementation: partial` ou `complete` exige
**uma decisão de engenharia por operação**, em `docs/decisoes/`.

Frontmatter obrigatório:

- `operation`: nome de domínio idêntico ao `CommandContract`.
- `implementation`: mesmo estado `partial` ou `complete`.
- `status`: `adopted-for-scope`, não aprovação universal.
- `scope`: recorte efetivamente implementado, sem generalizar.
- `selected_strategy`: alternativa adotada **somente para esse recorte**.
- `alternatives`: pelo menos duas estratégias distintas, incluindo a escolhida.
- `evidence_level`: grau de sustentação obtida.
- `live_validation`: ensaio live realizado ou não, sem inferir resultado.

As seções registram alternativas, comparação, escolha, links verificáveis
de contrato/evidência/teste e condições para rever a estratégia.
O gate não deve pontuar preferências tecnológicas nem forçar REST,
SOAP ou Web; exige justificativa concreta e permite decisões parciais.

É proibido descrever uma interface documentada como disponível na
instalação sem teste. Um resultado de replay offline não certifica
um fluxo HTTP live. Uma falha de escrita não permite fallback automático.
