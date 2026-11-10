---
type: SEIObservation
title: "GO: processo que exige validação de acesso antes de abrir a árvore"
description: "Leitura de 10/11/2026 no GO de treinamento: o processo 202600005006024 serve a árvore com src vazio e abre a modal usuario_validar_acesso; IES passa a reconhecer o caso com erro próprio, Todos também não abre o processo"
---

# GO: processo com validação de acesso — 10/11/2026

**Instalação:** `https://seitreinamento.go.gov.br/sei` (GO 5.0.3 de
**treinamento**), conta pública `aluno05`, unidade `SEAD/CDE-09516`.
**Processo:** `202600005006024`, primeiro Gerado visualizado da unidade.
Só leituras (GET); nenhuma escrita.

## O que o SEI serve

A tela `procedimento_trabalhar` vem com `<iframe id="ifrArvore" … src="">`
(**árvore vazia**) e o `inicializar()` da página chama
`infraAbrirJanelaModal('controlador.php?acao=usuario_validar_acesso&…')`: o SEI
pede que o usuário valide o acesso (senha) antes de carregar a árvore. Outros
três processos da mesma lista abriram normalmente.

## IES

Antes: `endereco_da_arvore_nao_reconhecido`, como se o HTML estivesse fora da
forma. Agora: erro próprio `ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso`,
com a mensagem "o SEI exige validar o acesso com a senha antes de abrir este
processo; o IES ainda não faz essa validação". Saída 1, sem nova requisição.
Árvore vazia **sem** a modal continua como endereço não reconhecido.
O diagnóstico usou um build local que gravou a tela em arquivo `0600` fora do
repositório; o arquivo foi apagado por conter `infra_hash`.

## Todos (oráculo)

`todos --json processo 202600005006024` (Todos `da13e85`, mesma conta) também
não abre o processo e responde `tela_inesperada`: "A árvore aberta não é a do
processo 202600005006024; recusando." O Todos não trata `usuario_validar_acesso`
(nenhuma ocorrência no código).

## Lacuna

Validar o acesso (enviar a senha na modal) é capacidade nova: precisa de issue
e BDD do 🐘. Até lá, o IES recusa com a causa correta.
