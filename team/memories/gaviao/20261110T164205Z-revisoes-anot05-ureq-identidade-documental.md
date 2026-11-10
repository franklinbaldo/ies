---
type: AgentMemory
title: "Revisões de ANOT-05, troca para ureq e recusa documental"
description: "Heads #680/#686 receberam LGTM técnico com CI 7/7 e sincronização pendente; #687 distingue hipótese HTTP de E2E e aguarda gates após Rustfmt."
agent: gaviao
timestamp: "2026-11-10T16:42:05Z"
tags: [revisao, bdd, anotacao, transporte, documento, cassettes, ci, cafezinho]
---

# Retomar

**Fato #680:** [parecer head `2038d11`](https://github.com/franklinbaldo/ies/pull/680#issuecomment-6773209683), LGTM BDD-ANOT-05, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42853670859). Quatro CLI reais, anotação com origem causal, SQLite e dois POSTs distintos contados. Preserva [#681](https://github.com/franklinbaldo/ies/pull/681), que mudou escrita não confirmada para saída 1/`nao_confirmado`. `behind=7` na comparação da rodada.

**Fato #686:** [parecer head `6850e19`](https://github.com/franklinbaldo/ies/pull/686#issuecomment-6773210638), [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42853613621); refatoração transversal HTTP/TLS/proxy para `ureq`, sem BDD novo. Risco externo live ainda não ensaiado nas sete instalações da #673. Comentário de dependência `base64` obsoleto em Cargo.toml, sugestão não bloqueante. `behind=2` na leitura.

**Fato #687:** [parecer head `bcf6833`](https://github.com/franklinbaldo/ies/pull/687#issuecomment-6773224264). Correção `id_documento` sem `=` na árvore anti-IES e teste HTTP negativo, hipótese sintética não observada nos cards UnB/GO/PEN. O CI rápido do primeiro head falhou em Rustfmt; o diff subsequente corrigiu os dois pontos, porém **sem CI verificado do SHA final** no encerramento. Teste técnico HTTP **não é** novo E2E CLI do BDD-60-05.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), em especial o [relato 🦊 sobre #681](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6772740711). A continuidade conceitual permanece: falha de confirmação não atesta ausência do efeito; saída 1 comunica estado não confirmado, sem autorizar retry. Isso não muda a classificação de cassette/live.

**Próxima rodada:** verificar novos SHAs e os gates de #687, reconciliar `main` antes de atribuir integração às #680/#686, sem repetir parecer para mesmo SHA. Não houve implementação funcional, merge, sessão, credencial ou acesso ao SEI pelo Gavião.
