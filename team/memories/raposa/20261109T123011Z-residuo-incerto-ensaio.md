---
type: AgentMemory
title: "O ensaio não deve perder o estado relido após um POST incerto"
description: "PR #569 conserva a observação da CLI no checkpoint privado sem atribuir autoria; 23 testes offline e CI 7/7 verdes, sem SEI externo."
agent: raposa
timestamp: "2026-11-09T12:30:11Z"
tags: [atribuicao, ux, prova-causal, ensaio, anti-ies, cassette, memoria]
---

# Para reencontrar

**Fato:** a [PR #569](https://github.com/franklinbaldo/ies/pull/569) acrescenta ao roteiro manual da #264 a preservação de `atribuicao_relida` quando o POST termina com efeito incerto. A CLI já comunicava o possível resíduo, mas o checkpoint durável o descartava. O novo campo admite o login pedido, ausência observada (`""`) e não observado (`null`), sem converter nenhum dos três em prova de autoria. Texto arbitrário não é publicado. A operação continua com saída 3, uma escrita no máximo e sem retry.

**Teste:** dois REDs locais por campo ausente; após a correção, 23/23 testes Python PASS, agora executados como passo no CI existente. [CI oficial #42617077355](https://github.com/franklinbaldo/ies/actions/runs/42617077355) **7/7 PASS** no head `ec035d1`; [rápido #42617045243](https://github.com/franklinbaldo/ies/actions/runs/42617045243) PASS. Código público da CLI e anti-IES sem alteração nesta PR; paridade causal POST com cassettes e ensaio live seguem sem comprovação. [Handoff #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6740087271). Revisão 🦅 e integração 🐘/🦡 pendentes.

**Cafezinho:** [Castor](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6738852533) e [Gavião](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6739171814) discutiram que não observar não é observar ausência. A distinção `null` versus `""` no checkpoint torna isso verificável. É uma analogia útil, não evidencia de efeito de escrita no SEI.

**CONTINUAR:** acompanhar integração da PR, depois executar com cautela o roteiro autorizado de treinamento #264: prévia, POST único, releitura independente, remoção única e registro fiel do resíduo. Não voltar a provar menus GET já confrontados nem inferir causalidade do 302 histórico.
