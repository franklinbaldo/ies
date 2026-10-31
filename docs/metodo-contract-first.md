---
type: Guide
title: Contract-First TDD e validação do IES
description: Ordem obrigatória de contrato, testes, implementação e validação reprodutível
---

# Contract-First TDD por operação

A sequência abaixo é obrigatória para mudanças de comportamento. Dividir o
trabalho em PRs pequenas é aceitável; uma PR com apenas contrato ou parser
não deve declarar a operação live completa. A issue de capacidade permanece
aberta até seus critérios de aceite terem sido demonstrados.

## CONTRACT — antes de codificar comportamento

1. Verificar o contrato OKF e a matriz de cassettes, por **operação e instalação**.
2. Definir entrada, saída, identidade opaca, efeitos e erros precisos em
   `enum` Rust, com subcódigos e campos tipados pertinentes. Jamais classificar
   erros pelo texto da mensagem.
3. Criar shims de assinatura e fronteira sem implementação falsa:
   `nao_implementado` explícito é preferível a um sucesso simulado.
4. Para qualquer contrato com `implementation: partial/complete`, registrar
   a decisão OKF de transporte, limites e comparação de alternativas.

## RED — testes antes do miolo

Escrever os testes usando os tipos públicos e as variantes específicas dos
erros, não comparações com mensagens de apresentação. Cobrir casos normais,
negativos, limites, identidade de instalação/unidade, acesso e efeitos.
Executar os testes, registrar a falha esperada e guardar a referência da
execução. Fixtures sintéticas são úteis para invariantes; evidência do SEI
exige cassette verificável, com revisão/hash exatos.

## GREEN / REFACTOR — comportamento e arquitetura

Implementar a menor lógica observável que satisfaça o contrato, preferindo
Web quando demonstrado. Eliminar requests duplicados com ablação controlada
e jamais remover uma verificação de efeito incerto. Rodar
`cargo test --locked --workspace --all-targets`, Clippy, rustfmt e o único
gate canônico `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`.
Refatorar com tudo verde; não aprovar PR com checks obrigatórios vermelhos.

## VALIDATED — múltiplas instalações, sem produção

Após os testes locais/offline, elaborar roteiro para UnB, UFRJ, Goiás e
PEN1–PEN4 que realmente estejam disponíveis e **autorizadas**. Para cada
combinação instalação + identidade + unidade + operação, registrar resultado,
variante de erro, versão declarada, efeito, tempo, quantidade de requisições
e diferença em relação ao contrato. Execução paralela é apropriada somente
para leituras sem efeito e estados isolados. GET de visualização pode registrar
recebimento; POST de paginação ou pesquisa exige análise de efeito.
Sem autorização, registrar `NOT_RUN` e o impedimento, sem tentar acesso.
Nunca usar instalação de produção como substituta.

## Handoff das cassettes

O [roteiro offline](../scripts/validar-cassettes.sh) executa cenários existentes
e verificados, isoladamente e em paralelo, **sem rede** pelo IES. Deve-se
preparar os ZIPs previamente, de acordo com cada card OKF. O script não
grava cassette nem publica no Internet Archive. O [roteiro de coleta privada](coleta-privada.md) prepara a execução delegada
por um coletor autorizado, sem publicar os dados brutos. O executor externo, com
autorização e coletor apropriados, deverá gravar os cenários live, excluir
cookies, URLs assinadas, dados pessoais e segredos da publicação, calcular
hashes e registrar manifesto/revisão. Nunca deixar o capturador no CI.
Reutilizar o roteiro por operação como entrada dos próximos testes offline.

## Relato de uma PR

Registrar `CONTRACT / RED / GREEN / VALIDATED`, cada evidência
`PASS / FAIL / NOT_RUN`, os comandos executados, a identidade da instalação
quando permitida, os efeitos observados e o limite entre offline e live.
`Refs #N` não fecha uma capacidade parcial; só usar `Closes #N` quando
todos os critérios do card forem atendidos.
