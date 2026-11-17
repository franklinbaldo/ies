---
type: SEIObservation
title: Estrutura da tela de Atualizar Andamento
description: Evidência de treinamento UnB sobre formulário e histórico posterior
---

# Atualizar Andamento

Fonte: Todos src/todos/andamento.py (commit 323eb01), observação UnB SEI 4.0.9 de 15/10/2026, e [cassette UnB](../../cassettes/unb/processo-andamento-registrar-unb--111df7e25e1e.md) com 23 requisições, cinco de procedimento_atualizar_andamento. O card não contém corpo de POST.

A barra publica procedimento_atualizar_andamento. A tela contém o form frmAtividadeListar com txaDescricao, hdnIdAtividade, hdnIdProtocolo e sbmSalvar. O SEI recusa descrição vazia ou maior de 500 caracteres, embora a tela não declare maxlength. Após escrita, o Todos observou 302 para procedimento_consultar_historico e nova linha no topo, normalizando quebras de linha em espaços.

Esta é evidência histórica do Todos, não validação ao vivo do novo comando IES. Nunca registrar URL assinada nem executar scripts da página.
