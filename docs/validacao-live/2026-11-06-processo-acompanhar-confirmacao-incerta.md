---
type: OperationEvidence
title: Acompanhamento especial — confirmação única incerta na UnB de treinamento
description: Ensaio autenticado de prévia, um POST e releitura independente sem repetição da escrita
operation: processo.acompanhar
---

# 🐢 Confirmação única com efeito incerto — UnB, 06/11/2026

## Fonte e limites

- Instalação: exclusivamente UnB **TREINAMENTO**, origem conferida contra a tabela pública `docs/interfaces/credenciais-de-treinamento.md`.
- Run do ensaio real: [42215793820](https://github.com/franklinbaldo/ies/actions/runs/42215793820), commit `2986cba`, com script temporário para uma única execução. Não confundir o status final **FAIL** do roteiro com ausência de POST.
- Processo de treinamento: `23106.001875/2026-93`, criado e identificado em ensaio histórico independente da #245. **Não** foi repetida a criação.
- Pedido: acompanhar no grupo padrão `""`, observação sintética `IES254 Tartaruga 2026-11-06 teste unico`.
- Credenciais e sessão em arquivos privados `0600` dentro de diretório `0700`. Nenhuma URL assinada, cookie, senha ou `infra_hash` foi publicada. Produção não acessada, JavaScript não executado, nenhum retry; pausa de 35 segundos entre comandos.
- Um workflow anterior, [42215635712](https://github.com/franklinbaldo/ies/actions/runs/42215635712), **FAIL antes da criação de jobs** por sintaxe YAML; **0 requisições** nessa tentativa, não é ensaio live. Corrigida a sintaxe antes do único roteiro de SEI.

## Resultado observado (não extrapolar o efeito)

| Etapa | Situação | Requisições publicadas | Observação |
| --- | --- | --- | --- |
| `ies login` | **PASS** (rc=0) | **6** | Sessão da UnB de treinamento |
| Prévia `processo acompanhar` sem `--confirmar` | **PASS** (rc=0) | **4** | `operacao=acompanhar`, `grupo=""`, `ja_estava=false`, `confirmado=false`, `alterado=false` |
| Único `--confirmar` | **FAIL / EFEITO INCERTO** (rc=3) | **não publicado pela CLI no erro** | `codigo=efeito_incerto`, `subcodigo=acompanhamento_nao_confirmado_por_leitura`. O fluxo de implementação chega a **um POST no máximo**, e esta saída só ocorre após uma tentativa de envio. **Não repetir o POST.** |
| Releitura independente, só leitura | **PASS** (rc=0) | **3** | Mesma origem e NUP; `grupo=""`, `ja_estava=true`, `alterado=false`. O script não registrou a observação da linha nem o ID, então a coincidência de grupo **não comprova a autoria nem o conteúdo exato da escrita**. |
| Resíduo | **PRESENTE, autoria incerta** | — | Grupo padrão aparece na releitura, onde não aparecia na prévia. Não é seguro concluir que o POST do IES o criou: a conta é compartilhada. ID e observação do registro não foram publicados. |

Total **conhecido** das três etapas com contagem: **6 + 4 + 3 = 13 requisições**; a etapa de confirmação executou pedidos adicionais cujo número não foi emitido no JSON de erro. Logo, **não declarar orçamento total exato**. Uma única invocação de `--confirmar` ocorreu. A escrita **não está comprovada** e nunca deve ser repetida neste NUP para esclarecer a dúvida.

O script primeiro exigiu prévia com processo, operação, grupo, observação, flags e orçamento coerentes; só então permitiu o único `--confirmar`. Depois realizou a releitura independente mesmo após rc=3. O script considerou o resultado geral **FAIL** por faltar prova do efeito; a existência posterior do grupo não converteu automaticamente rc=3 em sucesso.

## RED, CI, replay e continuidade

- **RED novo desta documentação/ensaio: NOT_RUN**, pois não houve mudança funcional em Rust nem testes novos. Não atribuir falhas que não ocorreram.
- RED histórico da idempotência do grupo padrão: [42208077518](https://github.com/franklinbaldo/ies/actions/runs/42208077518), **1 FAIL / 1.725 PASS / 23 ignorados preexistentes** antes da correção.
- O head anterior ao executor, `5d0759c`, teve [CI 42213622756](https://github.com/franklinbaldo/ies/actions/runs/42213622756) **6/6 PASS** (Rustfmt, Clippy, Rust, OKF, Release e quatro replays). **Conferir CI do head final** após publicar este relatório.
- O replay obrigatório de acompanhamento usa 8 cassettes de 7 instalações e 189 respostas; não comprova POST live.
- **`--deixar --confirmar` permanece recusado antes de sessão/rede**. Nenhuma retirada foi tentada. Outra instalação de treinamento: **NOT_RUN, 0 requisições**.

## Regressão funcional após a observação live

A leitura independente posterior à confirmação incerta encontrou grupo existente, mas a CLI publicava `id_acompanhamento: null` para inclusão idempotente, embora a tabela já tivesse uma identidade de linha. Pelo princípio **null = não observado**, isto perdia informação útil para reconciliar o caso, sem autorizar inferência sobre a autoria da escrita.

- Foram acrescentadas **duas asserções** aos testes sintéticos de grupo explícito e grupo padrão já acompanhado, exigindo `id_acompanhamento = "831"` lido na fixture, sem GET adicional ou POST.
- **RED observado antes do código:** [CI 42216759415](https://github.com/franklinbaldo/ies/actions/runs/42216759415), `prek` com **2 FAIL novos / 1.734 PASS / 23 ignored preexistentes**. Ambas falharam em `None != Some("831")`; Rustfmt e Clippy PASS.
- Correção mínima em `src/processo_acompanhar/fluxo.rs`: quando a lista comprova o grupo, retornar o ID **já observado** da linha, junto com sua observação; a operação continua idempotente, com 4 GET e **0 POST** nos testes. Contrato atualizado.
- **GREEN:** depende do CI do head final; não antecipar aprovação. Nenhuma escrita nova foi enviada ao SEI nesta correção.

**Próxima sessão:** investigar **somente por leitura**, sem refazer a escrita, a observação/ID da linha no grupo padrão e as evidências já disponíveis de redirecionamento e releitura; distinguir a recusa do aceite HTTP sem ampliar a confiança indevidamente. Para testar uma nova confirmação futura, usar outro processo de treinamento genuinamente não acompanhado e repetir a sequência apenas nesse novo alvo, com orçamento e resíduo verificáveis. Esta issue permanece WIP; não liberar retirada nem fechar.
