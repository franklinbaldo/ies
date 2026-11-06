---
type: OperationEvidence
title: Grupo padrão já acompanhado — observação UnB e regressão da issue 254
description: Ensaio de leitura autenticada no treinamento UnB e correção de idempotência sem POST
operation: processo.acompanhar
---

# 🐢 Grupo padrão de Acompanhamento Especial — 06/11/2026

**Observação antes do RED:** no [ensaio UnB treinamento 42207589575](https://github.com/franklinbaldo/ies/actions/runs/42207589575), a autenticação teve **PASS, 5 requisições**. Após pausa de 35 s, a prévia autenticada para o processo de treinamento `23106.001559/2026-11`, grupo padrão vazio, teve **PASS, 3 requisições**, `operacao=acompanhar`, `grupo=""`, `ja_estava=true`, `confirmado=false`, `alterado=false`. Total **8 requisições**, todas de login/leitura; **0 POST de acompanhamento**. A guarda do ensaio recusou a confirmação por o grupo já estar acompanhado. **Confirmação NOT_RUN, 0 POST; verificação pós-escrita NOT_RUN, 0 GET; nenhuma alteração ou resíduo novo**. O job encerrou com falha **intencional da guarda**, não da prévia. Sem retry e sem produção.

**Hipótese derivada da observação:** sem `--grupo` a escolha padrão da CLI é o grupo vazio; a otimização idempotente não pode ficar limitada a `Some("")`.

**RED real:** commit de teste `89c3732`, [CI 42208077518](https://github.com/franklinbaldo/ies/actions/runs/42208077518): **1 teste novo FAIL / 1.725 PASS / 23 ignorados anteriores**. Teste sintético `grupo_padrao_ja_acompanhado_nao_exige_cadastro_nem_post` exige sucesso já acompanhado, **4 GET e 0 POST**, sem URL de cadastro.

**GREEN candidato:** commit `450d939` ajusta apenas os três pontos de seleção/abertura do cadastro no prefixo `src/processo_acompanhar/fluxo.rs`, preservando o bloqueio público de `--deixar --confirmar`. O [CI 42208340262](https://github.com/franklinbaldo/ies/actions/runs/42208340262) comprovou a suíte Rust e Clippy PASS; somente Rustfmt pediu remover uma linha vazia no teste. Evidência de CI verde é exclusiva do head final.

**Pendente:** confirmação única em outra situação de treinamento ainda não acompanhada, após nova prévia; depois leitura independente e documentação do resíduo. Segunda instalação de treinamento não ensaiada nesta fatia; não repetir escrita de efeito incerto. Nenhuma capacidade declarada completa.
