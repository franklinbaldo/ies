---
name: cassette-e2e
type: Guide
title: "E2E via cassettes — teste de aceitação do IES"
description: "Testar a CLI Rust real com observações do SEI, anti-IES e SQLite, sem BDD, Gherkin ou Cucumber"
metadata:
  version: "1"
---

# E2E via cassettes

**Este é o método vigente desde 11/11/2026.** Não criar ".feature",
step definitions, cenários `Dado/Quando/Então`, runners Cucumber ou gates
que exigem aprovação prévia de BDD. Identificadores BDD em histórico de
issues são referências, nunca pré-condição de implementação.

1. **Contrato + cassette:** parta da operação existente em `contracts/cli/`
   e procure os cards em `cassettes/`, `docs/cassettes.md` e
   `contracts/evidencias/`. Confira versão, origem, ZIP/membro e SHA.
   Sem rede no sandbox, use o workflow `inspecionar-cassettes.yml` no
   GitHub Actions; **não invente captura disponível**.
2. **Leitor real:** rode o mesmo leitor do binário `ies` nas respostas
   originais (replay). Teste o resultado observável: identidade, corpo,
   status, opções, recusas, saída e arquivos. Não basta parser isolado.
3. **Transição:** quando houver escrita ou estado entre operações,
   construa SQLite efêmero versionado a partir da captura; execute a
   CLI real sobre HTTPS loopback → anti-IES → SQLite, confirme com outra
   invocação da CLI e o mesmo banco. Prévia: zero escrita; confirmação:
   no máximo uma, nenhuma repetição automática, releitura independente.
4. **Confronto bilateral:** compare **separadamente** IES e anti-IES
   com corpos originais e transições documentadas. Não use resposta
   sintética gerada pelo próprio anti-IES como oráculo de compatibilidade.
5. **Se não há captura relevante:** teste a hipótese mínima no E2E,
   identifique explicitamente a lacuna da cassette, não rotule PASS de
   replay. Não bloquee o desenvolvimento esperando REA/live.
6. **Live opcional, somente autorizado:** em SEI de treinamento, orçamento
   de requisições, prévia, confirmação única e verificação independente;
   agentes nunca operam produção. Nunca registrar credencial, cookie,
   URL assinada ou `infra_hash`.

## Resultado mínimo na PR

- Comando/contrato e card de cassette consultado (com hash, se disponível)
- E2E CLI real: PASS/FAIL/NOT_RUN, SHA, teste e cenário de estado
- Confronto independente IES e anti-IES ↔ resposta original: PASS/FAIL/NOT_RUN
- Ensaio em treinamento: PASS/FAIL/NOT_RUN e relatório, se executado
- Código/teste removido: justificativa, regressão preservada ou substituta

O teste sem cassette que apenas observa a saída do simulador é
**sintético**, não um teste de fidelidade. Testes unitários de Rust podem
permanecer quando exercitam invariantes não capturáveis.

Guia detalhado: [ciclo](../../../docs/anti-ies/ciclo-cassettes.md).
