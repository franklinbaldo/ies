---
type: AgentMemory
title: "🦊 #851: controle positivo CLI e negativo de ZIP no mesmo SHA"
description: "Ifro comprovou cinco casos sintéticos CLI/anti-IES/SQLite e triagem de seis editores originais; corrupção de ZIP foi detectada. Replay semântico bilateral e live não executados."
agent: raposa
timestamp: "2026-11-16T04:34:10Z"
tags: [ies, documento-salvar, cassette-e2e, cli, anti-ies, negative-control]
---

# Continuidade IES #851 · rodada de 16/11

**Fatos:** Na [PR #852 Draft](https://github.com/franklinbaldo/ies/pull/852), fonte HEAD `187a77dcf913385aed7ce07849c11eeec1cfbc15` no Ifro, após merge verdadeiro de `main@b92ad17c`. O teste Rust ignorado `replay_editor_nativo_no_acervo` com ZIPs originais UnB/GO foi executado offline: **1 PASS, 0 FAIL, 2231 filtered**, seis GETs nativos corretamente recusados pelo seletor sintético; este teste não salva no SEI.

**Controle positivo e negativo:** compilei e executei o binário da suíte `aies_documento_cli` em contêiner de ensaio, com CLI real, TLS loopback `sei.test`, anti-IES e SQLite efêmero: filtro legado `bdd_89`, **5 PASS, 0 FAIL, 28 filtered**, verificando publicação/versão, releitura, assinatura e duas recusas. O nome histórico do filtro não cria BDD novo. Em cópia isolada de ZIP UnB, adição de um byte fez o replay falhar corretamente com **RC=101, SHA-1 do ZIP não confere**. ZIPs originais preservados. O ensaio offline sem rede inicialmente bloqueou o túnel sintético; somente o resultado final em contêiner com loopback operacional é PASS.

**Separação de provas:** a CLI↔anti-IES↔SQLite **PASS somente sintético**. A triagem original IES de seis respostas **PASS limitada**, sem replay de transição ou semântica do POST. Comparação independente IES↔corpo original e anti-IES↔corpo original de salvamento **NOT_RUN**, pois os corpos completos dos pedidos POST não estão custodiados. Treinamento/live **NOT_RUN; nenhum GET/POST externo ao SEI**, nenhuma escrita real. CI rápido [#43803239325](https://github.com/franklinbaldo/ies/actions/runs/43803239325) SUCCESS em `187a77dc`, mas somente Rustfmt/Prek; CI oficial de sete jobs e replay no head a integrar **NOT_RUN**. Revisão 🦅 ainda pede a prova oficial.

**Casa:** snapshot pessoal exato anterior `raposa@a60bdcf4`, tree `3bb2f5bf`, 124 arquivos, pre-push instalado e Prek 2/2 PASS no Ifro; CI privado não é comprovado por isso. Sem novo código de produção, classificador defensivo ou mudança do protocolo externo. Manter Draft; após publicar esta memória na main, sincronizar a branch novamente e revalidar o SHA final.