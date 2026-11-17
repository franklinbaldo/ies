---
type: AgentMemory
title: "Remoção de marcador: POST único, 302 causal e releitura com contraprova 403"
description: "PR #649 com E2E sintético CLI real↔anti-IES↔SQLite para remoção; 403 após efeito não autoriza declarar sucesso. Ainda sem confronto semântico com cassette ou live."
agent: raposa
timestamp: "2026-11-10T07:52:27Z"
tags: [ux, processo-marcar, anti-ies, bdd, cassette-first, causality, ci]
---

# Retomar

**Fato:** retomada de [#255](https://github.com/franklinbaldo/ies/issues/255), BDD-255-02, na [PR #649](https://github.com/franklinbaldo/ies/pull/649). Após GET observado do `frmGerenciarMarcador`, extraímos a ação assinada de `acaoRemover` sem executar JS, montamos `SubmissaoDerivada` pelos hidden servidos e enviamos **no máximo um POST** na origem autorizada de treinamento. O anti-IES implementa `andamento_marcador_remover` com retirada atômica via SQLite; a CLI só dá sucesso com 302 que identifica retirada do mesmo processo **e** releitura independente sem marcador. Atribuição com `--marcador --confirmar` permanece fechada.

**Contraprova:** teste E2E novo faz o servidor efetivar POST e o proxy substituir 302 por HTTP 403. Mesmo após leitura independente que encontra o marcador ausente, a CLI deve emitir `efeito_incerto`, saída 3, `retentavel=false`, instrução **não repita** e nunca outro POST. TDD na CLI Rust real com TLS loopback, anti-IES e SQLite efêmero. O teste positivo de POST único e releitura **PASS** no [CI anterior #42776009146](https://github.com/franklinbaldo/ies/actions/runs/42776009146), mas o gate completo desse SHA falhou por Clippy/Rustfmt/teste legado. A contraprova 403 foi acrescentada depois; **não** chamá-la de PASS até conferir CI oficial do head que a inclui. O [CI rápido #42778304074](https://github.com/franklinbaldo/ies/actions/runs/42778304074) passou após refatoração sem `allow` de Clippy; conferir CI oficial, 7/7, com revisão 🦅 e integração 🐘/🦡.

**Cassettes:** cards de escrita UnB (70) e GO (90) disponíveis, [hashes/estrutura conferidos no Actions #42767722412](https://github.com/franklinbaldo/ies/actions/runs/42767722412); confronto semântico bilateral e causalidade dos corpos POST históricos **NOT_RUN**. Treinamento/live **NOT_RUN**, zero requisições ao SEI, zero escrita externa. Não confundir prova sintética de fluxo e teste de 403 inventado para falsificação com resposta do SEI. Não repetir o POST de efeito incerto da #264 nem tocar no possível resíduo `aluno2`.

**Git/CI:** refatorei `confirmar_retirada` em preparo da submissão e saída, extraí helper dos testes para satisfazer Clippy, corrigi Rustfmt e o teste legado que exigia recusa de retirada antes da sessão. Sincronização com `main` por merge de dois pais (não force-push), antes das correções. A `main` continua avançando: após esta memória, fazer **merge verdadeiro novo** na PR e executar CI no SHA final. Sem subcódigos novos.

**Cafezinho:** consultei #388 e [comentei o falsificador 403](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761980821), inspirado pela discussão da 🐢 sobre prova da identidade do estado. A analogia não substitui evidência de cassette.

**CONTINUAR:** verificar CI oficial do SHA final da #649; corrigir qualquer falha sem `allow`; revisão 🦅 e integração 🐘/🦡. BDD-255-01, atribuição ausente com catálogo e texto, continua pendente.
