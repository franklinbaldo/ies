---
type: SEIObservation
title: Replay offline do Acompanhamento Especial na issue 254
description: Respostas de sete instalações conferidas por hashes e leitores Rust reais sem acesso ao SEI
---

# Acompanhamento Especial — replay do acervo público

**Replay PASS:** [GitHub Actions 42170324775](https://github.com/franklinbaldo/ies/actions/runs/42170324775), job `replay acompanhamento especial (acervo público)`, em 06/11/2026. Teste Rust `processo_acompanhar::acervo::replay_do_acompanhamento_no_acervo`: 1 teste aprovado, nenhuma falha. Os três testes Python sintéticos do manifesto também passaram. Este run teve o job `prek` vermelho **somente por Rustfmt** no arquivo novo (corrigido depois); portanto o run inteiro **não** é anunciado como CI verde. Conferir o CI do head final.

Foram conferidas **8 cassettes** de UnB, UFRJ, Goiás, PEN1–4; **6 ZIPs** do acervo público do Internet Archive, com SHA-1; cada membro JSON passou por SHA-256, schema, completude e host da instalação via a peça compartilhada `cassette`. Nenhum endpoint do SEI foi acessado.

| Leitor real ou classe | Total | Lido | Recusado | Não aplicável |
|---|---:|---:|---:|---:|
| Lista de acompanhamentos | 82 | 82 | 0 | 0 |
| Cadastro de grupos | 45 | 45 | 0 | 0 |
| POST gravado, sem prova correlacionada do efeito | 39 | 0 | 0 | 39 |
| Outras respostas sem HTML utilizável | 23 | 0 | 0 | 23 |
| **Total** | **189** | **127** | **0** | **62** |

Os leitores **de produção** são `acompanhamentos_da_lista` e `grupos_do_cadastro`. O replay só fornece HTML **gravado**, não executa POST. Os 39 POSTs são tratados explicitamente como `post_nao_comprova_alteracao`: uma resposta HTTP isolada não comprova inclusão ou exclusão. A leitura das listas e dos cadastros é coberta em todas as sete instalações, sem inferir paridade de uma escrita.

O novo job permanente do `ci.yml` obtém as origens apenas dos cards `cassettes/*/processo-acompanhar-*.md`, recusa ZIPs fora do prefixo público e hashes inconsistentes, exige a existência do teste Rust ignorado e o executa com cache conferido. Reproduzir com os ZIPs no `IES_ACERVO_CACHE`:

```sh
python3 scripts/baixar_acervo_processo_acompanhar.py
scripts/replay processo_acompanhar::acervo::replay_do_acompanhamento_no_acervo
```

O downloader não acessa o SEI e não repete downloads de um ZIP já validado. O `#[ignore = "wip #254: ..."]` serve apenas para a suíte local sem ZIPs: no job dedicado esse teste é executado obrigatoriamente.

**Live em treinamento:** prévia NOT_RUN (0 GET), confirmação NOT_RUN (0 POST), verificação posterior NOT_RUN (0 GET); **0 requisições reais, 0 escritas e nenhum resíduo**. Não houve login nem uso da produção. A remoção `--deixar --confirmar` continua bloqueada **antes de abrir sessão ou rede**. O replay não substitui a sequência autorizada prévia → único `--confirmar` → conferência por leitura em instalação de treinamento.
