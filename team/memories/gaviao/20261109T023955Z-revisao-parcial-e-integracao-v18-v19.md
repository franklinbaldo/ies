---
type: AgentMemory
title: "Revisão da confirmação parcial e integração SQLite v18→v19"
description: "PR #528 tem POST local mais leitura da CLI, mas não confirmação via CLI; #521/v18 pronta; #526 comprova segunda consulta e aguarda migração v19 e CI no SHA."
agent: gaviao
timestamp: "2026-11-09T02:39:55Z"
tags: [revisao, bdd, anti-ies, sqlite, ci, cassettes, cafezinho]
---

# Para retomar

**Fatos:** publiquei pareceres dos novos heads [#528](https://github.com/franklinbaldo/ies/pull/528#issuecomment-6730930729) (`b27d29d`), [#521](https://github.com/franklinbaldo/ies/pull/521#issuecomment-6730931148) (`0dd9cec`) e [#526](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6730931592) (`5188493`). #528 tem CI 7/7 e POST no anti-IES com leitura posterior pela CLI, mas BDD-258-02 continua **PARCIAL**, pois a CLI não executa `--confirmar`; integração incremental é válida sem declarar cenário completo. #521 tem 7/7 no head, concilia a navegação da #525 e pode integrar como SQLite **v18**. #526 agora contém a segunda invocação real `ies processo NUP` que faltava; 7/7 foi no SHA `20fa54b`, enquanto o head `5188493` só teve CI rápido. Depois da #521, mover a criação para **v19**, conciliar `navegacao.rs` e obter CI integral no novo SHA.

**Evidência:** consultei os cards PEN1 de relacionamento/criação e UnB/GO de atribuição, além dos runs Actions de integridade. PASS sintético e verificação de hash não representam confronto semântico independente ou teste live; ambos permanecem NOT_RUN, sem bloquear fatias verdes.

**Cafezinho:** visitei a [conversa permanente #388](https://github.com/franklinbaldo/ies/issues/388), com 46 comentários. A ideia do contrafólio como observação independente permanece inspiração narrativa para distinguir causalidade de mera coincidência, não prova de comportamento SEI.

**Próximo passo:** verificar se #521 integrou e rever head da #526 depois de reconciliar v19. Não repetir pareceres nos três SHAs acima. Sem acesso ao SEI nem implementação ou merge de código.
