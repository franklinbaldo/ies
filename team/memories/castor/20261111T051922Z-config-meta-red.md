---
type: AgentMemory
title: "Contraprova RED de NotADirectory na configuração privada"
description: "PR #757 elimina MetadadosNaoLidos e reutiliza ErroDeIoNaLeitura(ErrorKind); RED Rust real 2079 PASS/1 FAIL, head reconciliado com CI completo pendente."
agent: castor
timestamp: "2026-11-11T05:19:22Z"
tags: [ies, castor, antipattern, configuracao, rust, causa-tipada, red-green]
---

# Para retomar

**Fato:** na [PR #757](https://github.com/franklinbaldo/ies/pull/757), a primeira consulta de metadados em `ler_configuracao_local` perdia `NotADirectory` ao usar `MetadadosNaoLidos`. O [RED oficial #42963428896](https://github.com/franklinbaldo/ies/actions/runs/42963428896) reproduziu o defeito exato: 2079 Rust PASS e 1 FAIL, com ancestral regular e sentinela sintética intacta. A correção mínima reutilizou `ErroDeIoNaLeitura(io::ErrorKind)`, sem tipo novo e sem mudar `NotFound`/recusas de segurança.

**Situação na escrita:** branch reconciliada por merge verdadeiro no head `09264e2`, [CI rápido #42964396149](https://github.com/franklinbaldo/ies/actions/runs/42964396149) PASS; [CI oficial #42964422554](https://github.com/franklinbaldo/ies/actions/runs/42964422554) ainda tinha seis jobs aprovados e `prek` em execução. Não declarar GREEN final antes da sétima verificação. Nenhum E2E novo, cassette SEI não aplicável à falha local, live NOT_RUN.

**Limite da fatia:** `ErroConfiguracao::ArquivoDeConfiguracaoNaoPodeSerLido` ainda converte a causa interna em unidade, preservando a classificação pública antiga. Alterar stderr/contrato requer coordenação com 🐘, não acrescentar wrapper disfarçado. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6787699174) registrei essa distinção.

**CONTINUAR:** conferir 7/7 do SHA `09264e2`, revisão 🦅, integração 🐘/🦡 e retomar outras ablações da [#719](https://github.com/franklinbaldo/ies/issues/719).